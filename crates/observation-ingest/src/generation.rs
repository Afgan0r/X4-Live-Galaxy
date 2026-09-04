use crate::batch_budget::{AggregateUsage, CandidateUsage};
use crate::candidate_limits::AcceptedVersions;
use crate::completion_types::{Candidate, StagedBatch};
use crate::model::AcceptedProjection;
use crate::{GenerationLimits, ReceiverDisposition};
use observation_domain::{BatchId, ImmutableBatchEnvelope, SectionKey, SectionStartEnvelope};
use sha2::{Digest, Sha256};
use std::collections::BTreeMap;
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
    #[must_use]
    pub fn candidate_expected_records(&self, key: &SectionKey) -> Option<usize> {
        self.candidates
            .get(key)
            .map(|candidate| candidate.start.expected_records)
    }
    #[must_use]
    pub fn candidate_manifest(&self, key: &SectionKey) -> Vec<(usize, &BatchId)> {
        self.candidates
            .get(key)
            .into_iter()
            .flat_map(|candidate| candidate.batches.values())
            .map(|batch| (batch.ordinal, &batch.envelope.batch_id))
            .collect()
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
            },
        );
        ReceiverDisposition::Received
    }
    pub fn stage_section_batch(
        &mut self,
        batch: ImmutableBatchEnvelope,
        canonical_bytes: &[u8],
        decoded_bytes: usize,
        work: usize,
        now: u64,
    ) -> ReceiverDisposition {
        let key = batch.section_key.clone();
        let digest: [u8; 32] = Sha256::digest(canonical_bytes).into();
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
            .map(|prior| prior.digest == digest)
        {
            Some(true) => return ReceiverDisposition::Received,
            Some(false) => return self.reject_candidate(&key),
            None => {}
        }
        if !self.versions_admit(&batch) {
            return self.reject_candidate(&key);
        }
        let delta = CandidateUsage {
            raw_bytes: canonical_bytes.len(),
            decoded_bytes,
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
        let ordinal = candidate.batches.len() + 1;
        candidate.batches.insert(
            batch.batch_id.clone(),
            StagedBatch {
                ordinal,
                digest,
                envelope: batch,
            },
        );
        candidate.usage = candidate_usage;
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
