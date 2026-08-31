use crate::batch::{apply_observation, apply_reconciliation_with_limit, complete_marker};
use crate::model::{AcceptedProjection, AdmissionError as Error, RejectionReason};
use crate::runtime_facts::RuntimeFacts;
use crate::snapshot::ProjectionSnapshot;
use crate::wire::{FrameHeader, WireFrame};
use crate::{GenerationLimits, GenerationProgress as Progress};
use observation_domain::{CanonicalObservationKey, EntityId};
use std::collections::BTreeMap;
struct Candidate {
    identity: (String, u64, u64),
    next_sequence: u64,
    staged_bytes: usize,
    work_units: usize,
    snapshot: ProjectionSnapshot,
    runtime_facts: BTreeMap<EntityId, RuntimeFacts>,
    observed: Vec<CanonicalObservationKey>,
}
pub struct GenerationStager {
    accepted: AcceptedProjection,
    limits: GenerationLimits,
    candidate: Option<Candidate>,
    last_admitted_generation: Option<u64>,
    admitted_generation_count: u64,
}
impl GenerationStager {
    #[must_use]
    pub const fn new(accepted: AcceptedProjection, limits: GenerationLimits) -> Self {
        Self {
            accepted,
            limits,
            candidate: None,
            last_admitted_generation: None,
            admitted_generation_count: 0,
        }
    }
    #[must_use]
    pub const fn resume(
        accepted: AcceptedProjection,
        limits: GenerationLimits,
        last_admitted_generation: u64,
    ) -> Self {
        let mut stager = Self::new(accepted, limits);
        stager.last_admitted_generation = Some(last_admitted_generation);
        stager
    }
    pub const fn accepted(&self) -> &AcceptedProjection {
        &self.accepted
    }
    #[must_use]
    pub const fn admitted_generation_count(&self) -> u64 {
        self.admitted_generation_count
    }
    pub fn stage_frame_at(&mut self, payload: &str, receipt: u64) -> Progress {
        match self.try_stage(payload, receipt) {
            Ok(progress) => progress,
            Err(error) => self.reject(&error),
        }
    }
    fn try_stage(&mut self, payload: &str, receipt: u64) -> Result<Progress, Error> {
        if receipt == 0 {
            return Err(Error::ReceiptClockUnavailable);
        }
        let header = crate::inspect_frame(payload)?;
        let FrameHeader::Data {
            scope,
            version,
            generation,
            sequence,
            ..
        } = header
        else {
            return Err(Error::InvalidFixture);
        };
        self.prepare(&scope, version, generation, sequence, payload.len())?;
        let frame = serde_json::from_str(payload).map_err(|_| Error::InvalidFixture)?;
        match frame {
            WireFrame::Observation(frame) => self.stage_observation(frame, receipt),
            WireFrame::CompleteMarker(frame) => self.commit_marker(frame, generation),
            _ => Err(Error::InvalidFixture),
        }
    }
    fn prepare(
        &mut self,
        scope: &str,
        version: u64,
        generation: u64,
        sequence: u64,
        bytes: usize,
    ) -> Result<(), Error> {
        if generation == 0
            || self
                .last_admitted_generation
                .is_some_and(|last| generation < last)
        {
            return Err(Error::OutOfOrderVersion);
        }
        if self.candidate.is_none() && sequence != 1 {
            return Err(Error::OutOfOrderVersion);
        }
        if self.candidate.is_none() {
            self.start_candidate(scope, version, generation);
        }
        let candidate = self.candidate.as_mut().ok_or(Error::InvalidFixture)?;
        if candidate.identity != (scope.to_owned(), version, generation)
            || candidate.next_sequence != sequence
        {
            return Err(Error::OutOfOrderVersion);
        }
        candidate.staged_bytes = candidate
            .staged_bytes
            .checked_add(bytes)
            .ok_or(Error::CollectionLimitExceeded)?;
        candidate.work_units = candidate
            .work_units
            .checked_add(1)
            .ok_or(Error::CollectionLimitExceeded)?;
        if candidate.staged_bytes > self.limits.max_staged_bytes
            || candidate.work_units > self.limits.max_work_units
        {
            return Err(Error::CollectionLimitExceeded);
        }
        candidate.next_sequence = sequence
            .checked_add(1)
            .ok_or(Error::CollectionLimitExceeded)?;
        Ok(())
    }
    fn start_candidate(&mut self, scope: &str, version: u64, generation: u64) {
        self.candidate = Some(Candidate {
            identity: (scope.to_owned(), version, generation),
            next_sequence: 1,
            staged_bytes: 0,
            work_units: 0,
            snapshot: self.accepted.snapshot.clone(),
            runtime_facts: self.accepted.runtime_facts.clone(),
            observed: Vec::new(),
        });
    }
    fn stage_observation(
        &mut self,
        frame: crate::wire::WireObservation,
        receipt: u64,
    ) -> Result<Progress, Error> {
        let candidate = self.candidate.as_mut().ok_or(Error::InvalidFixture)?;
        let (_, key, entity_id, facts) =
            apply_observation(&mut candidate.snapshot, frame, receipt)?;
        candidate.observed.push(key);
        candidate.runtime_facts.insert(entity_id, facts);
        Ok(Progress::Staged)
    }
    fn commit_marker(
        &mut self,
        frame: crate::wire::WireCompleteMarker,
        generation: u64,
    ) -> Result<Progress, Error> {
        let marker = complete_marker(frame)?;
        let mut candidate = self.candidate.take().ok_or(Error::InvalidFixture)?;
        let exact_replay = self
            .accepted
            .snapshot
            .completed_scope(marker.scope())
            .is_some_and(|scope| scope.is_exact_replay(marker.version(), &candidate.observed));
        apply_reconciliation_with_limit(
            &self.accepted.snapshot,
            &mut candidate.snapshot,
            &marker,
            &candidate.observed,
            candidate.observed.len().max(1),
        )?;
        candidate
            .runtime_facts
            .retain(|id, _| candidate.snapshot.observations.contains_key(id));
        self.accepted =
            AcceptedProjection::with_runtime_facts(candidate.snapshot, candidate.runtime_facts);
        if exact_replay || self.last_admitted_generation == Some(generation) {
            return Ok(Progress::Replay);
        }
        self.last_admitted_generation = Some(generation);
        self.admitted_generation_count = self
            .admitted_generation_count
            .checked_add(1)
            .ok_or(Error::CollectionLimitExceeded)?;
        Ok(Progress::Admitted)
    }
    fn reject(&mut self, error: &Error) -> Progress {
        self.candidate = None;
        let reason = RejectionReason::from(error);
        self.accepted = std::mem::take(&mut self.accepted).record_rejection(reason);
        Progress::Rejected(reason)
    }
}
