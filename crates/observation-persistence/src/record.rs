use std::fmt::Write;

use observation_domain::{
    CanonicalizationVersion, CompletionCoverage, DecisionSnapshotId, DigestAlgorithmVersion,
    EnvelopeRecord, ObservationPolicyVersion, ObservationSchemaVersion, SectionRevisionId,
};
use observation_ingest::{ContractVersions, DecisionRevisionSet};
use sha2::{Digest, Sha256};

use crate::{PersistedContext, PublicationLimits, PublishRequest, RevisionRecord};

pub fn normalize(request: &PublishRequest, limits: PublicationLimits) -> Option<RevisionRecord> {
    let revision = request.revision();
    if !request.is_authoritative()
        || revision.records().len() > limits.max_records.get()
        || revision.context().versions() != expected_versions()?
    {
        return None;
    }
    let bytes = revision
        .records()
        .iter()
        .try_fold(0usize, |total, record| {
            total.checked_add(record.content.len())
        })?;
    let calculated = content_digest(revision.records());
    if bytes > limits.max_content_bytes.get() || calculated != *revision.content_digest() {
        return None;
    }
    let mut record = RevisionRecord {
        source_scope: revision.source_scope().clone(),
        source_session: revision.source_session().clone(),
        section_key: revision.section_key().clone(),
        revision: revision.section_revision(),
        accepted_at: request.accepted_at(),
        records: revision.records().to_vec(),
        coverage: revision.coverage(),
        dependencies: request.frozen_dependencies().clone(),
        expected_current: request.expected_current(),
        manifest_digest: *revision.manifest_digest(),
        content_digest: calculated,
        integrity_digest: [0; 32],
        context: PersistedContext::from_candidate(revision.context()),
    };
    record.integrity_digest = integrity_digest(&record);
    Some(record)
}

#[must_use]
pub fn content_digest(records: &[EnvelopeRecord]) -> [u8; 32] {
    let mut digest = Sha256::new();
    for record in records {
        hash(&mut digest, record.record_id.as_str().as_bytes());
        hash(&mut digest, record.entity_id.as_str().as_bytes());
        hash(&mut digest, &record.observation_version.get().to_be_bytes());
        hash(&mut digest, record.content.as_bytes());
    }
    digest.finalize().into()
}

pub fn decision_identity(set: &DecisionRevisionSet) -> Option<DecisionSnapshotId> {
    let mut digest = Sha256::new();
    for (key, revision) in set.revisions() {
        hash(&mut digest, key.as_str().as_bytes());
        hash(&mut digest, &revision.get().to_be_bytes());
    }
    let mut value = String::from("decision:");
    for byte in digest.finalize() {
        write!(&mut value, "{byte:02x}").ok()?;
    }
    DecisionSnapshotId::new(value)
}

#[must_use]
pub fn integrity_digest(record: &RevisionRecord) -> [u8; 32] {
    let mut digest = Sha256::new();
    hash(&mut digest, record.source_scope.as_str().as_bytes());
    hash(
        &mut digest,
        record
            .source_session
            .producer_incarnation()
            .as_str()
            .as_bytes(),
    );
    hash(
        &mut digest,
        &record.source_session.transport_epoch().get().to_be_bytes(),
    );
    hash(&mut digest, record.section_key.as_str().as_bytes());
    hash(&mut digest, &record.revision.get().to_be_bytes());
    hash(&mut digest, &record.accepted_at.to_be_bytes());
    hash(&mut digest, &[coverage_byte(record.coverage)]);
    for (key, revision) in &record.dependencies {
        hash(&mut digest, key.as_str().as_bytes());
        hash(&mut digest, &revision.get().to_be_bytes());
    }
    hash(
        &mut digest,
        &record
            .expected_current
            .map_or(0, SectionRevisionId::get)
            .to_be_bytes(),
    );
    hash(&mut digest, &record.manifest_digest);
    hash(&mut digest, &record.content_digest);
    hash(&mut digest, record.context.canonical_payload().as_bytes());
    digest.finalize().into()
}

fn expected_versions() -> Option<ContractVersions> {
    Some(ContractVersions::new(
        ObservationSchemaVersion::new(1)?,
        ObservationPolicyVersion::new(2)?,
        CanonicalizationVersion::new(3)?,
        DigestAlgorithmVersion::new(1)?,
    ))
}

const fn coverage_byte(coverage: CompletionCoverage) -> u8 {
    match coverage {
        CompletionCoverage::Complete => 1,
        CompletionCoverage::KnownEmpty => 2,
        CompletionCoverage::Partial => 3,
        CompletionCoverage::Unknown => 4,
        CompletionCoverage::Unsupported => 5,
    }
}

fn hash(digest: &mut Sha256, value: &[u8]) {
    digest.update((value.len() as u64).to_be_bytes());
    digest.update(value);
}
