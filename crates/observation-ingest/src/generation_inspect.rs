use observation_domain::{BatchId, SectionKey};

use crate::GenerationStager;

impl GenerationStager {
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
}
