#![expect(
    clippy::expect_used,
    reason = "test fixtures fail immediately when their invariants are invalid"
)]

use std::collections::BTreeMap;

use observation_domain::{
    BatchId, CanonicalizationVersion, CaptureWindow, CompletionCoverage, DigestAlgorithmVersion,
    EntityId, EnvelopeRecord, ImmutableBatchEnvelope, ObservationPolicyVersion,
    ObservationSchemaVersion, ObservationVersion, ProducerIncarnationId, RecordId,
    SectionAvailability, SectionCompletionEnvelope, SectionCoverage, SectionFreshness, SectionKey,
    SectionQuality, SectionRevisionId, SectionStartEnvelope, SectionState, SourceScopeId,
    TransportEpoch,
};
use observation_ingest::{
    AcceptedProjection, AggregateLimits, CandidateContext, CandidateLimits, CompletionCurrent,
    CompletionOutcome, ContractVersions, GenerationLimits, GenerationStager, ReceiverDisposition,
    RejectionReason,
};

fn value<T>(raw: &str, make: impl FnOnce(String) -> Option<T>) -> T {
    make(raw.to_owned()).expect("fixture identity is valid")
}
const fn revision(raw: u64) -> SectionRevisionId {
    SectionRevisionId::new(raw).expect("revision is positive")
}
fn key(raw: &str) -> SectionKey {
    value(raw, SectionKey::new)
}
fn versions() -> ContractVersions {
    ContractVersions::new(
        ObservationSchemaVersion::new(1).expect("version is positive"),
        ObservationPolicyVersion::new(2).expect("version is positive"),
        CanonicalizationVersion::new(3).expect("version is positive"),
        DigestAlgorithmVersion::new(1).expect("version is positive"),
    )
}
fn limits() -> GenerationLimits {
    GenerationLimits::bounded(
        CandidateLimits::new(128, 256, 4, 4, 8, 100, 10).expect("limits are non-zero"),
        AggregateLimits::new(4, 512, 1024, 16, 16, 32).expect("limits are non-zero"),
    )
}
fn start() -> SectionStartEnvelope {
    SectionStartEnvelope {
        source_scope: value("scope:x4", SourceScopeId::new),
        producer_incarnation: value("producer:1", ProducerIncarnationId::new),
        transport_epoch: TransportEpoch::new(1).expect("epoch is positive"),
        section_key: key("ships"),
        section_revision: revision(7),
        expected_records: 2,
    }
}
fn context() -> CandidateContext {
    CandidateContext::new(
        versions(),
        CaptureWindow::new(10, 20).expect("window is ordered"),
        SectionState::with_evidence(
            CaptureWindow::new(10, 20).expect("window is ordered"),
            SectionFreshness::Fresh,
            SectionQuality::Fresh,
            SectionAvailability::Available,
            SectionCoverage::Complete,
        ),
        BTreeMap::from([(key("sectors"), revision(4))]),
        Some(revision(6)),
        true,
    )
}
fn batch(identity: &str, entity: &str, record: &str) -> ImmutableBatchEnvelope {
    ImmutableBatchEnvelope {
        source_scope: value("scope:x4", SourceScopeId::new),
        section_key: key("ships"),
        section_revision: revision(7),
        batch_id: value(identity, BatchId::new),
        records: vec![EnvelopeRecord {
            record_id: value(record, RecordId::new),
            entity_id: value(entity, EntityId::new),
            observation_version: ObservationVersion::new(1).expect("version is positive"),
            content: format!("content:{entity}"),
        }],
        optional_detail: None,
    }
}
fn completion() -> SectionCompletionEnvelope {
    SectionCompletionEnvelope {
        source_scope: value("scope:x4", SourceScopeId::new),
        section_key: key("ships"),
        section_revision: revision(7),
        record_count: 2,
        coverage: CompletionCoverage::Complete,
    }
}
fn current() -> CompletionCurrent {
    CompletionCurrent::new(
        BTreeMap::from([(key("sectors"), revision(4))]),
        Some(revision(6)),
    )
}
fn staged() -> GenerationStager {
    let mut stager = GenerationStager::new(AcceptedProjection::empty(), limits());
    assert_eq!(
        stager.start_section_with_context(start(), context(), 1),
        ReceiverDisposition::Received
    );
    assert_eq!(
        stager.stage_section_batch(
            batch("batch:1", "ship:2", "record:2"),
            b"batch-two",
            20,
            1,
            2
        ),
        ReceiverDisposition::Received
    );
    assert_eq!(
        stager.stage_section_batch(
            batch("batch:2", "ship:1", "record:1"),
            b"batch-one",
            20,
            1,
            3
        ),
        ReceiverDisposition::Received
    );
    stager
}

#[test]
fn exact_completion_yields_one_immutable_unpublished_revision() {
    let mut stager = staged();
    let certificate = stager
        .completion_certificate(completion())
        .expect("candidate exists");
    let CompletionOutcome::Validated(revision) =
        stager.complete_section(certificate, &current(), 4)
    else {
        panic!("exact completion must validate")
    };
    assert_eq!(revision.section_key(), &key("ships"));
    assert_eq!(revision.records()[0].record_id.as_str(), "record:1");
    assert_eq!(revision.records()[1].record_id.as_str(), "record:2");
    assert_eq!(revision.context(), &context());
    assert_eq!(stager.candidate_count(), 0);
}

#[test]
fn any_count_length_digest_or_version_mismatch_discards_candidate() {
    let mut stager = staged();
    let mut certificate = stager
        .completion_certificate(completion())
        .expect("candidate exists");
    certificate.record_count += 1;
    assert_eq!(
        stager.complete_section(certificate, &current(), 4),
        CompletionOutcome::Rejected(RejectionReason::CompletionMismatch)
    );
    assert_eq!(stager.candidate_count(), 0);
    assert_eq!(stager.aggregate_usage().candidate_count, 0);
}

#[test]
fn frozen_dependency_mismatch_discards_and_arms_finite_cooldown() {
    let mut stager = staged();
    let certificate = stager
        .completion_certificate(completion())
        .expect("candidate exists");
    let changed = CompletionCurrent::new(
        BTreeMap::from([(key("sectors"), revision(5))]),
        Some(revision(6)),
    );
    assert_eq!(
        stager.complete_section(certificate, &changed, 4),
        CompletionOutcome::Rejected(RejectionReason::DependencyChanged)
    );
    assert_eq!(
        stager.start_section_with_context(start(), context(), 5),
        ReceiverDisposition::TimedOutOrSuperseded
    );
    assert_eq!(
        stager.start_section_with_context(start(), context(), 15),
        ReceiverDisposition::Received
    );
}

#[test]
fn coverage_and_capture_evidence_are_preserved_without_publishing() {
    let mut stager = staged();
    let certificate = stager
        .completion_certificate(completion())
        .expect("candidate exists");
    let CompletionOutcome::Validated(revision) =
        stager.complete_section(certificate, &current(), 4)
    else {
        panic!("exact completion must validate")
    };
    assert_eq!(revision.coverage(), CompletionCoverage::Complete);
    assert_eq!(
        revision.context().capture_window(),
        CaptureWindow::new(10, 20).expect("window is ordered")
    );
    assert!(!revision.is_published());
}
