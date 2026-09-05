use crate::accepted_versions::AcceptedVersions;
use crate::batch_budget::{AggregateUsage, CandidateUsage};
use crate::batch_canonical::CanonicalBatch;
use crate::completion_types::{Candidate, StagedBatch};
use crate::model::AcceptedProjection;
use crate::{GenerationLimits, ReceiverDisposition};
use observation_domain::{ImmutableBatchEnvelope, SectionKey, SectionStartEnvelope};
use std::collections::BTreeMap;
#[derive(Clone)]
pub struct GenerationStager {
    pub(crate) accepted: AcceptedProjection,
    pub(crate) limits: GenerationLimits,
    pub(crate) candidates: BTreeMap<SectionKey, Candidate>,
    pub(crate) aggregate: AggregateUsage,
    pub accepted_versions: AcceptedVersions,
    pub(crate) cooldowns: BTreeMap<SectionKey, u64>,
    pub(crate) last_admitted_generation: Option<u64>,
    pub(crate) admitted_generation_count: u64,
}
impl GenerationStager {
    #[must_use]
    pub const fn new(accepted: AcceptedProjection, limits: GenerationLimits) -> Self {
        Self {
            accepted,
            limits,
            candidates: BTreeMap::new(),
            aggregate: AggregateUsage::ZERO,
            accepted_versions: BTreeMap::new(),
            cooldowns: BTreeMap::new(),
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
    #[must_use]
    pub fn candidate_count(&self) -> usize {
        self.candidates.len()
    }
    #[must_use]
    pub fn candidate_usage(&self, key: &SectionKey) -> Option<CandidateUsage> {
        self.candidates.get(key).map(|candidate| candidate.usage)
    }
    pub const fn aggregate_usage(&self) -> AggregateUsage {
        self.aggregate
    }
    pub fn start_section(&mut self, start: SectionStartEnvelope, now: u64) -> ReceiverDisposition {
        let key = start.section_key.clone();
        let existing = self
            .candidates
            .get(&key)
            .map(|current| current.start == start);
        match existing {
            Some(true) => return ReceiverDisposition::Received,
            Some(false) => return self.reject_candidate(&key),
            None => {}
        }
        let Some(next) = self.aggregate.add_candidate() else {
            return ReceiverDisposition::CapacityUnavailable;
        };
        if !next.within(self.limits.aggregate) {
            return ReceiverDisposition::CapacityUnavailable;
        }
        self.aggregate = next;
        self.candidates.insert(
            key,
            Candidate {
                start,
                usage: CandidateUsage::default(),
                started_at: now,
                last_progress_at: now,
                batches: BTreeMap::new(),
                legacy_identity: None,
                next_sequence: 1,
                legacy_frames: Vec::new(),
                context: None,
                provisional_versions: BTreeMap::new(),
            },
        );
        ReceiverDisposition::Received
    }
    pub fn stage_section_batch(
        &mut self,
        batch: ImmutableBatchEnvelope,
        work: usize,
        now: u64,
    ) -> ReceiverDisposition {
        let key = batch.section_key.clone();
        let Some(canonical) = CanonicalBatch::from_envelope(&batch) else {
            return self.reject_candidate(&key);
        };
        let Some(candidate) = self.candidates.get(&key) else {
            return ReceiverDisposition::PermanentlyRejected;
        };
        if candidate.start.producer_incarnation != batch.producer_incarnation
            || candidate.start.transport_epoch != batch.transport_epoch
        {
            return ReceiverDisposition::StaleEpoch;
        }
        if candidate.start.source_scope != batch.source_scope
            || candidate.start.section_revision != batch.section_revision
        {
            return self.reject_candidate(&key);
        }
        match candidate
            .batches
            .get(&batch.batch_id)
            .map(|prior| prior.canonical_bytes == canonical.bytes)
        {
            Some(true) => return ReceiverDisposition::Received,
            Some(false) => return self.reject_candidate(&key),
            None => {}
        }
        if batch.section_ordinal == 0 || batch.section_ordinal != candidate.batches.len() + 1 {
            return self.reject_candidate(&key);
        }
        let Some(provisional_versions) = self.provisional_versions(candidate, &batch) else {
            return self.reject_candidate(&key);
        };
        let delta = CandidateUsage {
            raw_bytes: canonical.bytes.len(),
            decoded_bytes: canonical.decoded_bytes,
            records: batch.records.len(),
            batches: 1,
            work,
        };
        let Some((candidate_usage, aggregate)) = self.checked_charge(candidate, delta) else {
            return self.reject_candidate(&key);
        };
        let Some(candidate) = self.candidates.get_mut(&key) else {
            return ReceiverDisposition::PermanentlyRejected;
        };
        let ordinal = batch.section_ordinal;
        candidate.batches.insert(
            batch.batch_id.clone(),
            StagedBatch {
                ordinal,
                canonical_bytes: canonical.bytes,
                digest: canonical.digest,
                envelope: batch,
            },
        );
        candidate.usage = candidate_usage;
        candidate.provisional_versions = provisional_versions;
        candidate.last_progress_at = now;
        self.aggregate = aggregate;
        ReceiverDisposition::Received
    }
    pub(crate) fn drop_candidate(&mut self, key: &SectionKey) -> Option<Candidate> {
        let candidate = self.candidates.remove(key)?;
        self.aggregate = self.aggregate.release(candidate.usage)?;
        Some(candidate)
    }
    fn reject_candidate(&mut self, key: &SectionKey) -> ReceiverDisposition {
        self.drop_candidate(key);
        ReceiverDisposition::PermanentlyRejected
    }
    fn checked_charge(
        &self,
        candidate: &Candidate,
        delta: CandidateUsage,
    ) -> Option<(CandidateUsage, AggregateUsage)> {
        let candidate_usage = candidate.usage.charged(
            delta.raw_bytes,
            delta.decoded_bytes,
            delta.records,
            delta.work,
        )?;
        let aggregate = self.aggregate.add(delta)?;
        candidate_usage
            .within(self.limits.candidate)
            .then_some(())
            .and_then(|()| aggregate.within(self.limits.aggregate).then_some(()))
            .map(|()| (candidate_usage, aggregate))
    }
}
