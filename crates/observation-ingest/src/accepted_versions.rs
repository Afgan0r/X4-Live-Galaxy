use std::collections::BTreeMap;

use observation_domain::{EntityId, ImmutableBatchEnvelope, ObservationVersion, SourceScopeId};
use sha2::{Digest, Sha256};

use crate::{
    GenerationStager, HydratedSectionRevision, ValidatedSectionRevision,
    completion_types::Candidate,
};

pub type AcceptedVersions = BTreeMap<(SourceScopeId, EntityId), (ObservationVersion, [u8; 32])>;

impl GenerationStager {
    pub fn record_committed_revision(&mut self, revision: &ValidatedSectionRevision) {
        for record in revision.records() {
            let digest = Sha256::digest(record.content.as_bytes()).into();
            self.accepted_versions.insert(
                (revision.source_scope().clone(), record.entity_id.clone()),
                (record.observation_version, digest),
            );
        }
    }

    pub fn restore_committed_fence<'a>(
        &mut self,
        revisions: impl IntoIterator<Item = &'a ValidatedSectionRevision>,
    ) -> bool {
        let mut restored = AcceptedVersions::new();
        let valid = revisions
            .into_iter()
            .all(|revision| merge_revision(&mut restored, revision));
        if valid {
            self.accepted_versions = restored;
        }
        valid
    }

    pub fn restore_hydrated_fence<'a>(
        &mut self,
        revisions: impl IntoIterator<Item = &'a HydratedSectionRevision>,
    ) -> bool {
        let mut restored = AcceptedVersions::new();
        let valid = revisions.into_iter().all(|revision| {
            merge_records(&mut restored, revision.source_scope(), revision.records())
        });
        if valid {
            self.accepted_versions = restored;
        }
        valid
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

    pub(crate) fn provisional_versions(
        &self,
        candidate: &Candidate,
        batch: &ImmutableBatchEnvelope,
    ) -> Option<AcceptedVersions> {
        batch.records.iter().try_fold(
            candidate.provisional_versions.clone(),
            |mut provisional, record| {
                record_admits(
                    &self.accepted_versions,
                    &mut provisional,
                    &batch.source_scope,
                    record,
                )
                .then_some(provisional)
            },
        )
    }

    pub(crate) fn candidate_versions_still_admit(&self, candidate: &Candidate) -> bool {
        candidate
            .provisional_versions
            .iter()
            .all(|(key, (observed_version, observed_digest))| {
                version_digest_admits(
                    self.accepted_versions.get(key),
                    *observed_version,
                    *observed_digest,
                )
            })
    }
}

fn merge_revision(restored: &mut AcceptedVersions, revision: &ValidatedSectionRevision) -> bool {
    merge_records(restored, revision.source_scope(), revision.records())
}

fn merge_records(
    restored: &mut AcceptedVersions,
    scope: &SourceScopeId,
    records: &[observation_domain::EnvelopeRecord],
) -> bool {
    records
        .iter()
        .all(|record| merge_restored(restored, scope, record))
}

fn merge_restored(
    restored: &mut AcceptedVersions,
    scope: &SourceScopeId,
    record: &observation_domain::EnvelopeRecord,
) -> bool {
    let key = (scope.clone(), record.entity_id.clone());
    let value = (
        record.observation_version,
        Sha256::digest(record.content.as_bytes()).into(),
    );
    match restored.get(&key) {
        Some((version, digest)) if value.0 == *version => value.1 == *digest,
        Some((version, _)) if value.0 < *version => true,
        _ => {
            restored.insert(key, value);
            true
        }
    }
}

fn record_admits(
    accepted_versions: &AcceptedVersions,
    provisional: &mut AcceptedVersions,
    scope: &SourceScopeId,
    record: &observation_domain::EnvelopeRecord,
) -> bool {
    let key = (scope.clone(), record.entity_id.clone());
    if !version_admits(
        accepted_versions.get(&key),
        record.observation_version,
        record.content.as_bytes(),
    ) {
        return false;
    }
    let value = (
        record.observation_version,
        Sha256::digest(record.content.as_bytes()).into(),
    );
    provisional
        .insert(key, value)
        .is_none_or(|prior| prior == value)
}

fn version_admits(
    accepted: Option<&(ObservationVersion, [u8; 32])>,
    observed_version: ObservationVersion,
    content: &[u8],
) -> bool {
    let observed_digest = Sha256::digest(content).into();
    version_digest_admits(accepted, observed_version, observed_digest)
}

fn version_digest_admits(
    accepted: Option<&(ObservationVersion, [u8; 32])>,
    observed_version: ObservationVersion,
    observed_digest: [u8; 32],
) -> bool {
    match accepted {
        Some((version, _)) if observed_version < *version => false,
        Some((version, digest)) if observed_version == *version => observed_digest == *digest,
        _ => true,
    }
}

#[cfg(test)]
#[path = "accepted_versions_tests.rs"]
mod tests;
