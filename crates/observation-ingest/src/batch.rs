use crate::batch_budget::BatchBudget;
use crate::model::{
    AcceptedProjection, AdmissionError, AdmissionOutcome, RejectionEvidence, RejectionReason,
};
use crate::runtime_facts::RuntimeFacts;
use crate::snapshot::{ProjectionSnapshot, ScopedObservation};
use crate::wire::{WireCompleteMarker, WireFrame, WireObservation};
use crate::{ReceiptClock, SystemReceiptClock, validate_health, validate_telemetry};
use observation_domain::{
    CanonicalObservationKey, CollectionLimit, CompleteMarker, DuplicateDecision, EntityId,
    ObservationRecord, ObservationSource, ObservationTime, ObservationVersion,
    ReconciliationDecision, SectionQuality, classify_duplicate, reconcile_membership,
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
        ReconciliationDecision::PreservedIncompleteScope
        | ReconciliationDecision::AwaitingSecondAbsence { .. } => Ok(()),
    }
}
