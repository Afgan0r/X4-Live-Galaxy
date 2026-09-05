use std::collections::BTreeMap;

use observation_domain::{
    CanonicalizationVersion, CaptureWindow, CompletionCoverage, DigestAlgorithmVersion, EntityId,
    EnvelopeRecord, ObservationPolicyVersion, ObservationSchemaVersion, ObservationVersion,
    ProducerIncarnationId, RecordId, SectionAvailability, SectionCoverage, SectionFreshness,
    SectionKey, SectionQuality, SectionRevisionId, SectionState, SourceScopeId,
    SourceSessionIdentity, TransportEpoch,
};

use crate::{
    AcceptedProjection, AggregateLimits, CandidateContext, CandidateLimits, ContractVersions,
    GenerationLimits, GenerationStager, ValidatedSectionRevision,
};

#[test]
fn restore_uses_the_highest_entity_version_independent_of_section_order() {
    for revisions in [
        vec![revision("alpha", 2, "new"), revision("beta", 1, "old")],
        vec![revision("beta", 1, "old"), revision("alpha", 2, "new")],
    ] {
        let mut stager = stager();
        assert!(stager.restore_committed_fence(&revisions));
        assert!(!stager.record_accepted_entity(scope(), entity(), version(1), b"old"));
        assert!(stager.record_accepted_entity(scope(), entity(), version(2), b"new"));
    }
}

#[test]
fn restore_rejects_equal_version_content_conflict_without_mutation() {
    let mut stager = stager();
    assert!(stager.record_accepted_entity(scope(), entity(), version(3), b"existing"));
    assert!(
        !stager
            .restore_committed_fence(
                &[revision("alpha", 2, "left"), revision("beta", 2, "right"),]
            )
    );
    assert!(!stager.record_accepted_entity(scope(), entity(), version(2), b"left"));
}

fn revision(section: &str, entity_version: u64, content: &str) -> ValidatedSectionRevision {
    ValidatedSectionRevision {
        source_scope: scope(),
        source_session: SourceSessionIdentity::new(
            ProducerIncarnationId::new("producer:1").expect("producer is valid"),
            TransportEpoch::new(1).expect("epoch is non-zero"),
        ),
        section_key: SectionKey::new(section).expect("section is valid"),
        section_revision: SectionRevisionId::new(1).expect("revision is non-zero"),
        records: vec![EnvelopeRecord {
            record_id: RecordId::new(format!("record:{section}")).expect("record is valid"),
            entity_id: entity(),
            observation_version: version(entity_version),
            content: content.to_owned(),
        }],
        coverage: CompletionCoverage::Complete,
        context: context(),
        manifest_digest: [0; 32],
        content_digest: [0; 32],
    }
}

fn stager() -> GenerationStager {
    GenerationStager::new(
        AcceptedProjection::empty(),
        GenerationLimits::bounded(
            CandidateLimits::new(1, 1, 1, 1, 1, 1, 1).expect("limits are non-zero"),
            AggregateLimits::new(1, 1, 1, 1, 1, 1).expect("limits are non-zero"),
        ),
    )
}

fn context() -> CandidateContext {
    let window = CaptureWindow::new(1, 2).expect("window is ordered");
    CandidateContext::new(
        ContractVersions::new(
            ObservationSchemaVersion::new(1).expect("version is non-zero"),
            ObservationPolicyVersion::new(2).expect("version is non-zero"),
            CanonicalizationVersion::new(3).expect("version is non-zero"),
            DigestAlgorithmVersion::new(1).expect("version is non-zero"),
        ),
        window,
        SectionState::with_evidence(
            window,
            SectionFreshness::Fresh,
            SectionQuality::Fresh,
            SectionAvailability::Available,
            SectionCoverage::Complete,
        ),
        BTreeMap::new(),
        None,
        true,
    )
}

fn scope() -> SourceScopeId {
    SourceScopeId::new("scope:x4").expect("scope is valid")
}
fn entity() -> EntityId {
    EntityId::new("ship:shared").expect("entity is valid")
}
const fn version(value: u64) -> ObservationVersion {
    ObservationVersion::new(value).expect("version is non-zero")
}
