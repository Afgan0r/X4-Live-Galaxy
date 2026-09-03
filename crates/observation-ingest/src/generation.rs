use std::collections::BTreeMap;

use observation_domain::{
    BatchId, EntityId, ImmutableBatchEnvelope, ObservationVersion, SectionKey, SectionRevisionId,
    SectionStartEnvelope, SourceScopeId,
};
use sha2::{Digest, Sha256};

use crate::batch_budget::{AggregateUsage, CandidateUsage};
use crate::model::AcceptedProjection;
use crate::wire::WireObservation;
use crate::{GenerationLimits, ReceiverDisposition};

#[derive(Clone)]
pub(crate) struct StagedBatch {
    pub(crate) ordinal: usize,
    pub(crate) digest: [u8; 32],
    pub(crate) envelope: ImmutableBatchEnvelope,
}

pub(crate) struct Candidate {
    pub(crate) source_scope: SourceScopeId,
    pub(crate) revision: SectionRevisionId,
    pub(crate) expected_records: usize,
    pub(crate) usage: CandidateUsage,
    pub(crate) started_at: u64,
    pub(crate) last_progress_at: u64,
    pub(crate) batches: BTreeMap<BatchId, StagedBatch>,
    pub(crate) legacy_identity: Option<(String, u64, u64)>,
    pub(crate) next_sequence: u64,
    pub(crate) legacy_frames: Vec<(WireObservation, u64)>,
}

pub struct GenerationStager {
    pub(crate) accepted: AcceptedProjection,
    pub(crate) limits: GenerationLimits,
    pub(crate) candidates: BTreeMap<SectionKey, Candidate>,
    pub(crate) aggregate: AggregateUsage,
    accepted_versions: BTreeMap<(SourceScopeId, EntityId), (ObservationVersion, [u8; 32])>,
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
            aggregate: AggregateUsage {
                candidate_count: 0,
                raw_bytes: 0,
                decoded_bytes: 0,
                records: 0,
                batches: 0,
                work: 0,
            },
            accepted_versions: BTreeMap::new(),
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
            .map(|candidate| candidate.expected_records)
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
    #[must_use]
    pub const fn aggregate_usage(&self) -> AggregateUsage {
        self.aggregate
    }

    pub fn record_accepted_entity(
        &mut self,
        scope: SourceScopeId,
        entity: EntityId,
        version: ObservationVersion,
        canonical_content: &[u8],
    ) -> bool {
        let digest: [u8; 32] = Sha256::digest(canonical_content).into();
        match self.accepted_versions.get(&(scope.clone(), entity.clone())) {
            Some((current, _)) if version < *current => false,
            Some((current, prior)) if version == *current && prior != &digest => false,
            _ => {
                self.accepted_versions
                    .insert((scope, entity), (version, digest));
                true
            }
        }
    }

    pub fn start_section(&mut self, start: SectionStartEnvelope, now: u64) -> ReceiverDisposition {
        let key = start.section_key.clone();
        if let Some(current) = self.candidates.get(&key) {
            if current.source_scope == start.source_scope
                && current.revision == start.section_revision
            {
                return ReceiverDisposition::Received;
            }
            self.drop_candidate(&key);
            return ReceiverDisposition::PermanentlyRejected;
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
                source_scope: start.source_scope,
                revision: start.section_revision,
                expected_records: start.expected_records,
                usage: CandidateUsage::default(),
                started_at: now,
                last_progress_at: now,
                batches: BTreeMap::new(),
                legacy_identity: None,
                next_sequence: 1,
                legacy_frames: Vec::new(),
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
        if candidate.source_scope != batch.source_scope
            || candidate.revision != batch.section_revision
        {
            self.drop_candidate(&key);
            return ReceiverDisposition::PermanentlyRejected;
        }
        if let Some(prior) = candidate.batches.get(&batch.batch_id) {
            if prior.digest == digest {
                return ReceiverDisposition::Received;
            }
            self.drop_candidate(&key);
            return ReceiverDisposition::PermanentlyRejected;
        }
        if !self.versions_admit(&batch) {
            self.drop_candidate(&key);
            return ReceiverDisposition::PermanentlyRejected;
        }
        let delta = CandidateUsage {
            raw_bytes: canonical_bytes.len(),
            decoded_bytes,
            records: batch.records.len(),
            batches: 1,
            work,
        };
        let Some(candidate_usage) = candidate.usage.charged(
            delta.raw_bytes,
            delta.decoded_bytes,
            delta.records,
            delta.work,
        ) else {
            self.drop_candidate(&key);
            return ReceiverDisposition::PermanentlyRejected;
        };
        let Some(aggregate) = self.aggregate.add(delta) else {
            self.drop_candidate(&key);
            return ReceiverDisposition::PermanentlyRejected;
        };
        if !candidate_usage.within(self.limits.candidate)
            || !aggregate.within(self.limits.aggregate)
        {
            self.drop_candidate(&key);
            return ReceiverDisposition::PermanentlyRejected;
        }
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

    pub fn expire_candidates(&mut self, now: u64) -> usize {
        let keys: Vec<_> = self
            .candidates
            .iter()
            .filter(|(_, candidate)| {
                now.saturating_sub(candidate.started_at) > self.limits.candidate.age_millis.get()
                    || now.saturating_sub(candidate.last_progress_at)
                        > self.limits.candidate.inactivity_millis.get()
            })
            .map(|(key, _)| key.clone())
            .collect();
        for key in &keys {
            self.drop_candidate(key);
        }
        keys.len()
    }

    fn versions_admit(&self, batch: &ImmutableBatchEnvelope) -> bool {
        batch.records.iter().all(|record| {
            match self
                .accepted_versions
                .get(&(batch.source_scope.clone(), record.entity_id.clone()))
            {
                Some((version, _)) if record.observation_version < *version => false,
                Some((version, digest)) if record.observation_version == *version => {
                    <[u8; 32]>::from(Sha256::digest(record.content.as_bytes())) == *digest
                }
                _ => true,
            }
        })
    }

    pub(crate) fn drop_candidate(&mut self, key: &SectionKey) -> Option<Candidate> {
        let candidate = self.candidates.remove(key)?;
        self.aggregate = self.aggregate.release(candidate.usage)?;
        Some(candidate)
    }
}
