use crate::batch::{apply_observation, apply_reconciliation_with_limit, complete_marker};
use crate::completion_types::Candidate;
use crate::wire::{FrameHeader, WireCompleteMarker, WireFrame};
use crate::{
    AcceptedProjection, AdmissionError, CandidateUsage, GenerationProgress, GenerationStager,
    RejectionReason,
};
use observation_domain::{SectionKey, SectionRevisionId, SourceScopeId};
use std::collections::BTreeMap;
type LegacyResult = Result<GenerationProgress, AdmissionError>;
type HeaderResult = Result<(SectionKey, u64, u64, u64), AdmissionError>;
impl GenerationStager {
    pub fn stage_frame_at(&mut self, payload: &str, receipt: u64) -> GenerationProgress {
        let key = crate::inspect_frame(payload)
            .ok()
            .and_then(|header| match header {
                FrameHeader::Data { scope, .. } => SectionKey::new(scope),
                FrameHeader::Hello { .. } => None,
            });
        self.try_stage(payload, receipt)
            .unwrap_or_else(|error| self.reject(key.as_ref(), &error))
    }
    fn reject(&mut self, key: Option<&SectionKey>, error: &AdmissionError) -> GenerationProgress {
        if let Some(key) = key {
            self.drop_candidate(key);
        }
        let reason = RejectionReason::from(error);
        self.accepted = std::mem::take(&mut self.accepted).record_rejection(reason);
        GenerationProgress::Rejected(reason)
    }
    fn try_stage(&mut self, payload: &str, receipt: u64) -> LegacyResult {
        let (key, version, generation, sequence) = self.header(payload, receipt)?;
        if !self.candidates.contains_key(&key) {
            self.begin_legacy_candidate(&key, version, generation, sequence, receipt)?;
        }
        self.stage_legacy_payload(&key, payload, generation, sequence, receipt)
    }
    fn header(&self, payload: &str, receipt: u64) -> HeaderResult {
        if receipt == 0 {
            return Err(AdmissionError::ReceiptClockUnavailable);
        }
        let FrameHeader::Data {
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
        let key = SectionKey::new(scope).ok_or(AdmissionError::InvalidScope)?;
        Ok((key, version, generation, sequence))
    }
    fn begin_legacy_candidate(
        &mut self,
        key: &SectionKey,
        version: u64,
        generation: u64,
        sequence: u64,
        receipt: u64,
    ) -> Result<(), AdmissionError> {
        if sequence != 1 {
            return Err(AdmissionError::OutOfOrderVersion);
        }
        let aggregate = self
            .aggregate
            .add_candidate()
            .filter(|usage| usage.within(self.limits.aggregate))
            .ok_or(AdmissionError::CollectionLimitExceeded)?;
        let source_scope =
            SourceScopeId::new(key.as_str().to_owned()).ok_or(AdmissionError::InvalidScope)?;
        let revision = SectionRevisionId::new(version).ok_or(AdmissionError::InvalidVersion)?;
        self.aggregate = aggregate;
        self.candidates.insert(
            key.clone(),
            Candidate {
                source_scope,
                revision,
                expected_records: 0,
                usage: CandidateUsage::default(),
                started_at: receipt,
                last_progress_at: receipt,
                batches: BTreeMap::new(),
                legacy_identity: Some((key.as_str().to_owned(), version, generation)),
                next_sequence: 1,
                legacy_frames: Vec::new(),
                context: None,
            },
        );
        Ok(())
    }
    fn stage_legacy_payload(
        &mut self,
        key: &SectionKey,
        payload: &str,
        generation: u64,
        sequence: u64,
        receipt: u64,
    ) -> LegacyResult {
        let candidate = self
            .candidates
            .get(key)
            .ok_or(AdmissionError::InvalidFixture)?;
        let identity = (
            key.as_str().to_owned(),
            candidate.revision.get(),
            generation,
        );
        if candidate.legacy_identity != Some(identity) || candidate.next_sequence != sequence {
            return Err(AdmissionError::OutOfOrderVersion);
        }
        let delta = CandidateUsage {
            raw_bytes: payload.len(),
            decoded_bytes: payload.len(),
            records: 0,
            batches: 1,
            work: 1,
        };
        let usage = candidate
            .usage
            .charged(delta.raw_bytes, delta.decoded_bytes, 0, 1)
            .filter(|usage| {
                usage.raw_bytes <= self.limits.max_staged_bytes
                    && usage.work <= self.limits.max_work_units
            })
            .ok_or(AdmissionError::CollectionLimitExceeded)?;
        let aggregate = self
            .aggregate
            .add(delta)
            .filter(|usage| usage.within(self.limits.aggregate))
            .ok_or(AdmissionError::CollectionLimitExceeded)?;
        let candidate = self
            .candidates
            .get_mut(key)
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
            WireFrame::CompleteMarker(frame) => self.commit(key, frame, generation),
            _ => Err(AdmissionError::InvalidFixture),
        }
    }
    fn commit(
        &mut self,
        key: &SectionKey,
        frame: WireCompleteMarker,
        generation: u64,
    ) -> LegacyResult {
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
