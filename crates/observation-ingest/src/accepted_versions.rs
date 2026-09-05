use std::collections::BTreeMap;

use observation_domain::{EntityId, ImmutableBatchEnvelope, ObservationVersion, SourceScopeId};
use sha2::{Digest, Sha256};

use crate::{GenerationStager, ValidatedSectionRevision};

pub type AcceptedVersions = BTreeMap<(SourceScopeId, EntityId), (ObservationVersion, [u8; 32])>;

impl GenerationStager {
    pub fn record_committed_revision(&mut self, revision: &ValidatedSectionRevision) {
        for record in revision.records() {
            let _ = self.record_accepted_entity(
                revision.source_scope().clone(),
                record.entity_id.clone(),
                record.observation_version,
                record.content.as_bytes(),
            );
        }
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

    pub(crate) fn versions_admit(&self, batch: &ImmutableBatchEnvelope) -> bool {
        batch.records.iter().all(|record| {
            let accepted = self
                .accepted_versions
                .get(&(batch.source_scope.clone(), record.entity_id.clone()));
            version_admits(
                accepted,
                record.observation_version,
                record.content.as_bytes(),
            )
        })
    }
}

fn version_admits(
    accepted: Option<&(ObservationVersion, [u8; 32])>,
    observed_version: ObservationVersion,
    content: &[u8],
) -> bool {
    match accepted {
        Some((version, _)) if observed_version < *version => false,
        Some((version, digest)) if observed_version == *version => {
            let observed: [u8; 32] = Sha256::digest(content).into();
            observed == *digest
        }
        _ => true,
    }
}
