use crate::batch_budget::BatchBudget;
use crate::generation::Candidate;
use crate::model::{
    AcceptedProjection, AdmissionError, AdmissionOutcome, RejectionEvidence, RejectionReason,
};
use crate::runtime_facts::RuntimeFacts;
use crate::snapshot::{ProjectionSnapshot, ScopedObservation};
use crate::wire::{WireCompleteMarker, WireFrame, WireObservation};
use crate::{
    GenerationProgress, GenerationStager, ReceiptClock, SystemReceiptClock, validate_health,
    validate_telemetry,
};
use observation_domain::{
    CanonicalObservationKey, CollectionLimit, CompleteMarker, DuplicateDecision, EntityId,
    ObservationRecord, ObservationSource, ObservationTime, ObservationVersion,
    ReconciliationDecision, SectionKey, SectionQuality, SectionRevisionId, SourceScopeId,
    classify_duplicate, reconcile_membership,
};
use std::collections::BTreeMap;
const MAX_TRACER_PAYLOAD_BYTES: usize = 2_048;
const MAX_SCOPE_MEMBERS: usize = 64;
pub const MAX_BATCH_FRAMES: usize = 128;
pub const MAX_BATCH_BYTES: usize = MAX_BATCH_FRAMES * MAX_TRACER_PAYLOAD_BYTES;
pub const MAX_BATCH_SCOPES: usize = 16;
pub const MAX_BATCH_MARKERS: usize = MAX_BATCH_SCOPES;
pub const MAX_BATCH_OBSERVATIONS: usize = MAX_BATCH_FRAMES;
pub fn validate_batch(
    accepted: &AcceptedProjection,
    frames: &[&str],
) -> Result<ProjectionSnapshot, AdmissionError> {
    Ok(validate_runtime_batch(accepted, frames, 1)?.0)
}

impl GenerationStager {
    pub fn stage_frame_at(&mut self, payload: &str, receipt: u64) -> GenerationProgress {
        let key = crate::inspect_frame(payload)
            .ok()
            .and_then(|header| match header {
                crate::wire::FrameHeader::Data { scope, .. } => SectionKey::new(scope),
                crate::wire::FrameHeader::Hello { .. } => None,
            });
        match self.try_stage_legacy(payload, receipt) {
            Ok(progress) => progress,
            Err(error) => {
                if let Some(key) = key {
                    let _ = self.drop_candidate(&key);
                }
                let reason = RejectionReason::from(&error);
                self.accepted = std::mem::take(&mut self.accepted).record_rejection(reason);
                GenerationProgress::Rejected(reason)
            }
        }
    }

    fn try_stage_legacy(
        &mut self,
        payload: &str,
        receipt: u64,
    ) -> Result<GenerationProgress, AdmissionError> {
        if receipt == 0 {
            return Err(AdmissionError::ReceiptClockUnavailable);
        }
        let crate::wire::FrameHeader::Data {
            scope,
            version,
            generation,
            sequence,
            ..
        } = crate::inspect_frame(payload)?
        else {
            return Err(AdmissionError::InvalidFixture);
        };
        if generation == 0
            || self
                .last_admitted_generation
                .is_some_and(|last| generation < last)
        {
            return Err(AdmissionError::OutOfOrderVersion);
        }
        let key = SectionKey::new(scope.clone()).ok_or(AdmissionError::InvalidScope)?;
        if !self.candidates.contains_key(&key) {
            if sequence != 1 {
                return Err(AdmissionError::OutOfOrderVersion);
            }
            let aggregate = self
                .aggregate
                .add_candidate()
                .ok_or(AdmissionError::CollectionLimitExceeded)?;
            if !aggregate.within(self.limits.aggregate) {
                return Err(AdmissionError::CollectionLimitExceeded);
            }
            self.aggregate = aggregate;
            self.candidates.insert(
                key.clone(),
                Candidate {
                    source_scope: SourceScopeId::new(scope.clone())
                        .ok_or(AdmissionError::InvalidScope)?,
                    revision: SectionRevisionId::new(version)
                        .ok_or(AdmissionError::InvalidVersion)?,
                    expected_records: 0,
                    usage: crate::CandidateUsage::default(),
                    started_at: receipt,
                    last_progress_at: receipt,
                    batches: BTreeMap::new(),
                    legacy_identity: Some((scope.clone(), version, generation)),
                    next_sequence: 1,
                    legacy_frames: Vec::new(),
                    context: None,
                },
            );
        }
        let candidate = self
            .candidates
            .get(&key)
            .ok_or(AdmissionError::InvalidFixture)?;
        if candidate.legacy_identity != Some((scope, version, generation))
            || candidate.next_sequence != sequence
        {
            return Err(AdmissionError::OutOfOrderVersion);
        }
        let usage = candidate
            .usage
            .charged(payload.len(), payload.len(), 0, 1)
            .ok_or(AdmissionError::CollectionLimitExceeded)?;
        let delta = crate::CandidateUsage {
            raw_bytes: payload.len(),
            decoded_bytes: payload.len(),
            records: 0,
            batches: 1,
            work: 1,
        };
        let aggregate = self
            .aggregate
            .add(delta)
            .ok_or(AdmissionError::CollectionLimitExceeded)?;
        if usage.raw_bytes > self.limits.max_staged_bytes
            || usage.work > self.limits.max_work_units
            || !aggregate.within(self.limits.aggregate)
        {
            return Err(AdmissionError::CollectionLimitExceeded);
        }
        let candidate = self
            .candidates
            .get_mut(&key)
            .ok_or(AdmissionError::InvalidFixture)?;
        candidate.usage = usage;
        candidate.next_sequence = sequence
            .checked_add(1)
            .ok_or(AdmissionError::CollectionLimitExceeded)?;
        self.aggregate = aggregate;
        match serde_json::from_str(payload).map_err(|_| AdmissionError::InvalidFixture)? {
            WireFrame::Observation(frame) => {
                candidate.legacy_frames.push((frame, receipt));
                Ok(GenerationProgress::Staged)
            }
            WireFrame::CompleteMarker(frame) => self.commit_legacy(&key, frame, generation),
            _ => Err(AdmissionError::InvalidFixture),
        }
    }

    fn commit_legacy(
        &mut self,
        key: &SectionKey,
        frame: WireCompleteMarker,
        generation: u64,
    ) -> Result<GenerationProgress, AdmissionError> {
        let marker = complete_marker(frame)?;
        let candidate = self
            .drop_candidate(key)
            .ok_or(AdmissionError::InvalidFixture)?;
        let mut snapshot = self.accepted.snapshot.clone();
        let mut runtime_facts = self.accepted.runtime_facts.clone();
        let mut observed = Vec::new();
        for (frame, receipt) in candidate.legacy_frames {
            let (_, observation, entity, facts) = apply_observation(&mut snapshot, frame, receipt)?;
            observed.push(observation);
            runtime_facts.insert(entity, facts);
        }
        let replay = self
            .accepted
            .snapshot
            .completed_scope(marker.scope())
            .is_some_and(|scope| scope.is_exact_replay(marker.version(), &observed));
        apply_reconciliation_with_limit(
            &self.accepted.snapshot,
            &mut snapshot,
            &marker,
            &observed,
            observed.len().max(1),
        )?;
        runtime_facts.retain(|id, _| snapshot.observations.contains_key(id));
        self.accepted = AcceptedProjection::with_runtime_facts(snapshot, runtime_facts);
        if replay || self.last_admitted_generation == Some(generation) {
            return Ok(GenerationProgress::Replay);
        }
        self.last_admitted_generation = Some(generation);
        self.admitted_generation_count = self
            .admitted_generation_count
            .checked_add(1)
            .ok_or(AdmissionError::CollectionLimitExceeded)?;
        Ok(GenerationProgress::Admitted)
    }
}
fn validate_runtime_batch(
    accepted: &AcceptedProjection,
    frames: &[&str],
    receipt_unix_millis: u64,
) -> Result<(ProjectionSnapshot, BTreeMap<EntityId, RuntimeFacts>), AdmissionError> {
    let mut candidate = accepted.snapshot.clone();
    let mut runtime_facts = accepted.runtime_facts.clone();
    let mut markers = Vec::new();
    let mut observed_members = BTreeMap::<EntityId, Vec<CanonicalObservationKey>>::new();
    let mut budget = BatchBudget::new(frames.len())?;
    for frame in frames {
        budget.record_frame(frame.len())?;
        if frame.len() > MAX_TRACER_PAYLOAD_BYTES {
            return Err(AdmissionError::FrameTooLarge);
        }
        match serde_json::from_str(frame).map_err(|_| AdmissionError::InvalidFixture)? {
            WireFrame::Hello(_) => return Err(AdmissionError::InvalidFixture),
            WireFrame::Heartbeat(frame) => validate_telemetry(frame.scope, frame.version)?,
            WireFrame::RuntimeHealth(frame) => validate_health(frame)?,
            WireFrame::Observation(frame) => {
                budget.record_observation()?;
                let (scope, key, entity_id, facts) =
                    apply_observation(&mut candidate, frame, receipt_unix_millis)?;
                budget.register_scope(&scope)?;
                observed_members.entry(scope).or_default().push(key);
                runtime_facts.insert(entity_id, facts);
            }
            WireFrame::CompleteMarker(frame) => {
                budget.record_marker()?;
                let marker = complete_marker(frame)?;
                budget.register_scope(marker.scope())?;
                markers.push(marker);
            }
        }
    }
    for marker in &markers {
        let observed = observed_members
            .get(marker.scope())
            .map(Vec::as_slice)
            .unwrap_or_default();
        apply_reconciliation_with_limit(
            &accepted.snapshot,
            &mut candidate,
            marker,
            observed,
            MAX_SCOPE_MEMBERS,
        )?;
    }
    runtime_facts.retain(|entity_id, _| candidate.observations.contains_key(entity_id));
    Ok((candidate, runtime_facts))
}
pub fn admit_batch(accepted: AcceptedProjection, frames: &[&str]) -> AdmissionOutcome {
    admit_batch_with_receipt_clock(accepted, frames, &SystemReceiptClock)
}
pub fn admit_batch_with_receipt_clock(
    accepted: AcceptedProjection,
    frames: &[&str],
    clock: &dyn ReceiptClock,
) -> AdmissionOutcome {
    let outcome = validate_runtime_batch(&accepted, frames, 1).and_then(|_| {
        let receipt = clock.receipt_unix_millis()?;
        (receipt > 0)
            .then_some(receipt)
            .ok_or(AdmissionError::ReceiptClockUnavailable)
            .and_then(|receipt| validate_runtime_batch(&accepted, frames, receipt))
    });
    match outcome {
        Ok((snapshot, runtime_facts)) => AdmissionOutcome::Accepted(
            AcceptedProjection::with_runtime_facts(snapshot, runtime_facts),
        ),
        Err(error) => {
            let evidence = RejectionEvidence::new(RejectionReason::from(&error));
            AdmissionOutcome::Rejected {
                projection: accepted.record_rejection(evidence.reason()),
                evidence,
            }
        }
    }
}
pub fn apply_observation(
    candidate: &mut ProjectionSnapshot,
    frame: WireObservation,
    receipt_unix_millis: u64,
) -> Result<(EntityId, CanonicalObservationKey, EntityId, RuntimeFacts), AdmissionError> {
    let entity_id = EntityId::new(frame.entity_id).ok_or(AdmissionError::InvalidEntityId)?;
    let scope = EntityId::new(frame.scope).ok_or(AdmissionError::InvalidScope)?;
    let version = ObservationVersion::new(frame.version).ok_or(AdmissionError::InvalidVersion)?;
    let quality = SectionQuality::from(frame.quality);
    let record = ObservationRecord::new(
        entity_id.clone(),
        ObservationSource::x4_runtime(),
        ObservationTime::from_unix_millis(receipt_unix_millis),
        version,
        "runtime_facts_v2".to_owned(),
    )
    .map_err(|_| AdmissionError::InvalidContent)?;
    let mut facts = frame.runtime_facts;
    facts.validate(&entity_id)?;
    facts.receipt_unix_millis = receipt_unix_millis;
    let key = CanonicalObservationKey::new(entity_id.clone(), version);
    if let Some(previous) = candidate.observations.get(&entity_id) {
        match classify_duplicate(&previous.record, &record) {
            DuplicateDecision::Idempotent => {
                return Ok((scope, key, entity_id, facts));
            }
            DuplicateDecision::Conflict if version == previous.record.version() => {
                return Err(AdmissionError::EqualVersionConflict);
            }
            DuplicateDecision::Conflict if version < previous.record.version() => {
                return Err(AdmissionError::OutOfOrderVersion);
            }
            DuplicateDecision::Conflict => {}
            DuplicateDecision::DifferentIdentity => return Err(AdmissionError::InvalidEntityId),
        }
    }
    candidate.observations.insert(
        entity_id.clone(),
        ScopedObservation {
            scope: scope.clone(),
            record,
            quality,
        },
    );
    Ok((scope, key, entity_id, facts))
}
pub fn complete_marker(frame: WireCompleteMarker) -> Result<CompleteMarker, AdmissionError> {
    let scope = EntityId::new(frame.scope).ok_or(AdmissionError::InvalidScope)?;
    let version = ObservationVersion::new(frame.version).ok_or(AdmissionError::InvalidVersion)?;
    Ok(CompleteMarker::successful(scope, version))
}
pub fn apply_reconciliation_with_limit(
    accepted: &ProjectionSnapshot,
    candidate: &mut ProjectionSnapshot,
    marker: &CompleteMarker,
    observed: &[CanonicalObservationKey],
    member_limit: usize,
) -> Result<(), AdmissionError> {
    let scope = marker.scope();
    if observed.iter().any(|key| key.version() != marker.version()) {
        return Err(AdmissionError::OutOfOrderVersion);
    }
    if let Some(completed) = accepted.completed_scope(scope) {
        if marker.version() < completed.version() {
            return Err(AdmissionError::OutOfOrderVersion);
        }
        if marker.version() == completed.version() {
            return completed
                .is_exact_replay(marker.version(), observed)
                .then_some(())
                .ok_or(AdmissionError::OutOfOrderVersion);
        }
    }
    let limit =
        CollectionLimit::new(member_limit).ok_or(AdmissionError::CollectionLimitExceeded)?;
    match reconcile_membership(
        &accepted.keys_for_scope(scope),
        observed.to_vec(),
        scope,
        Some(marker),
        limit,
    ) {
        ReconciliationDecision::Reconciled { tombstones, .. } => {
            candidate.remove_tombstones(scope, &tombstones);
            candidate.record_completion(scope.clone(), marker.version(), observed.to_vec());
            Ok(())
        }
        ReconciliationDecision::RejectedCollectionLimit => {
            Err(AdmissionError::CollectionLimitExceeded)
        }
        ReconciliationDecision::PreservedIncompleteScope => Ok(()),
    }
}
