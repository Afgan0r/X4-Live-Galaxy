#![expect(clippy::expect_used, reason = "invalid test fixtures fail immediately")]

use observation_domain::{
    CanonicalizationVersion, CaptureWindow, CompletionCoverage, DigestAlgorithmVersion,
    ImmutableBatchEnvelope, ObservationPolicyVersion, ObservationSchemaVersion,
    ProducerIncarnationId, SectionCompletionEnvelope, SectionCoverage, SectionFreshness,
    SectionKey, SectionRevisionId, SectionStartEnvelope, SectionState, SourceScopeId,
    TransportEpoch,
};
use observation_ingest::{
    AcceptedProjection, AggregateLimits, CandidateContext, CandidateLimits, CompletionCurrent,
    CompletionOutcome, ContractVersions, GenerationLimits, GenerationStager, ReceiverDisposition,
    RejectionReason,
};
use std::collections::BTreeMap;

fn id<T>(raw: &str, make: impl FnOnce(String) -> Option<T>) -> T {
    make(raw.to_owned()).expect("fixture identity is valid")
}

const fn revision() -> SectionRevisionId {
    SectionRevisionId::new(1).expect("revision is positive")
}

fn start() -> SectionStartEnvelope {
    SectionStartEnvelope {
        source_scope: id("scope:x4", SourceScopeId::new),
        producer_incarnation: id("producer:1", ProducerIncarnationId::new),
        transport_epoch: TransportEpoch::new(1).expect("epoch is positive"),
        section_key: id("ships", SectionKey::new),
        section_revision: revision(),
        expected_records: 0,
    }
}

fn limits() -> GenerationLimits {
    GenerationLimits::bounded(
        CandidateLimits::new(64, 64, 1, 1, 1, 100, 10).expect("limits are non-zero"),
        AggregateLimits::new(1, 64, 64, 1, 1, 1).expect("limits are non-zero"),
    )
}

const fn context() -> CandidateContext {
    CandidateContext::new(
        ContractVersions::new(
            ObservationSchemaVersion::new(1).expect("version is positive"),
            ObservationPolicyVersion::new(1).expect("version is positive"),
            CanonicalizationVersion::new(1).expect("version is positive"),
            DigestAlgorithmVersion::new(1).expect("version is positive"),
        ),
        CaptureWindow::new(1, 2).expect("window is ordered"),
        SectionState::new(SectionFreshness::Fresh, SectionCoverage::Complete),
        BTreeMap::new(),
        None,
        true,
    )
}

#[test]
fn changed_start_metadata_is_a_terminal_identity_conflict() {
    for mutate in [
        |start: &mut SectionStartEnvelope| {
            start.producer_incarnation = id("producer:2", ProducerIncarnationId::new);
        },
        |start: &mut SectionStartEnvelope| start.expected_records = 1,
    ] {
        let mut stager = GenerationStager::new(AcceptedProjection::empty(), limits());
        assert_eq!(
            stager.start_section(start(), 1),
            ReceiverDisposition::Received
        );
        let mut changed = start();
        mutate(&mut changed);
        assert_eq!(
            stager.start_section(changed, 2),
            ReceiverDisposition::PermanentlyRejected
        );
        assert_eq!(stager.candidate_count(), 0);
    }
}

#[test]
fn stale_batch_session_is_fenced_without_consuming_current_candidate() {
    let mut stager = GenerationStager::new(AcceptedProjection::empty(), limits());
    let start = start();
    assert_eq!(
        stager.start_section(start.clone(), 1),
        ReceiverDisposition::Received
    );
    let mut batch = ImmutableBatchEnvelope {
        source_scope: start.source_scope,
        producer_incarnation: id("producer:old", ProducerIncarnationId::new),
        transport_epoch: start.transport_epoch,
        section_key: start.section_key,
        section_revision: start.section_revision,
        batch_id: id("batch:1", observation_domain::BatchId::new),
        section_ordinal: 1,
        records: Vec::new(),
        optional_detail: None,
    };
    assert_eq!(
        stager.stage_section_batch(batch.clone(), 1, 2),
        ReceiverDisposition::StaleEpoch
    );
    assert_eq!(stager.candidate_count(), 1);
    batch.producer_incarnation = id("producer:1", ProducerIncarnationId::new);
    batch.transport_epoch = TransportEpoch::new(2).expect("epoch is positive");
    assert_eq!(
        stager.stage_section_batch(batch, 1, 3),
        ReceiverDisposition::StaleEpoch
    );
    assert_eq!(stager.candidate_count(), 1);
}

#[test]
fn stale_completion_session_cannot_validate_current_candidate() {
    let mut stager = GenerationStager::new(AcceptedProjection::empty(), limits());
    let start = start();
    assert_eq!(
        stager.start_section_with_context(start.clone(), context(), 1),
        ReceiverDisposition::Received
    );
    let envelope = SectionCompletionEnvelope {
        source_scope: start.source_scope,
        producer_incarnation: id("producer:old", ProducerIncarnationId::new),
        transport_epoch: start.transport_epoch,
        section_key: start.section_key,
        section_revision: start.section_revision,
        batch_count: 0,
        record_count: 0,
        raw_bytes: 0,
        decoded_bytes: 0,
        ordered_batch_manifest_digest: [0; 32],
        canonical_content_digest: [0; 32],
        schema_version: ObservationSchemaVersion::new(1).expect("version is positive"),
        policy_version: ObservationPolicyVersion::new(1).expect("version is positive"),
        canonicalization_version: CanonicalizationVersion::new(1).expect("version is positive"),
        digest_version: DigestAlgorithmVersion::new(1).expect("version is positive"),
        coverage: CompletionCoverage::Complete,
    };
    let envelope =
        observation_ingest::bind_completion_certificate(envelope, &[], context().versions())
            .expect("producer certificate binds");
    let certificate = stager
        .completion_certificate(envelope)
        .expect("candidate exists");
    assert_eq!(
        stager.complete_section(
            &certificate,
            &CompletionCurrent::new(BTreeMap::new(), None),
            2
        ),
        CompletionOutcome::Rejected(RejectionReason::CompletionMismatch)
    );
    assert_eq!(stager.candidate_count(), 0);
}
