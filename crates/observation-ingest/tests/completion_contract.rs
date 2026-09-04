#![expect(clippy::expect_used, reason = "invalid test fixtures fail immediately")]
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
    CompletionOutcome, ContractVersions, DecisionEligibility, DecisionRevisionIndex,
    EligibilityBlocker, GenerationLimits, GenerationStager, ReceiverDisposition, RejectionReason,
};
use std::collections::BTreeMap;
#[path = "completion_contract/rejections.rs"]
mod rejections;
fn value<T>(raw: &str, make: impl FnOnce(String) -> Option<T>) -> T {
    make(raw.to_owned()).expect("fixture identity is valid")
}
const fn revision(raw: u64) -> SectionRevisionId {
    SectionRevisionId::new(raw).expect("revision is positive")
}
fn key(raw: &str) -> SectionKey {
    value(raw, SectionKey::new)
}
const fn versions() -> ContractVersions {
    ContractVersions::new(
        ObservationSchemaVersion::new(1).expect("version is positive"),
        ObservationPolicyVersion::new(2).expect("version is positive"),
        CanonicalizationVersion::new(3).expect("version is positive"),
        DigestAlgorithmVersion::new(1).expect("version is positive"),
    )
}
fn limits() -> GenerationLimits {
    GenerationLimits::bounded(
        CandidateLimits::new(2_048, 4_096, 4, 4, 8, 100, 10).expect("limits are non-zero"),
        AggregateLimits::new(4, 8_192, 16_384, 16, 16, 32).expect("limits are non-zero"),
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
        producer_incarnation: value("producer:1", ProducerIncarnationId::new),
        transport_epoch: TransportEpoch::new(1).expect("epoch is positive"),
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
        producer_incarnation: value("producer:1", ProducerIncarnationId::new),
        transport_epoch: TransportEpoch::new(1).expect("epoch is positive"),
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
    let batches = [
        (batch("batch:1", "ship:2", "record:2"), 2),
        (batch("batch:2", "ship:1", "record:1"), 3),
    ];
    for (batch, now) in batches {
        assert_eq!(
            stager.stage_section_batch(batch, 1, now),
            ReceiverDisposition::Received
        );
    }
    stager
}
fn finish(stager: &mut GenerationStager) -> observation_ingest::ValidatedSectionRevision {
    let certificate = stager
        .completion_certificate(completion())
        .expect("candidate exists");
    let revision = match stager.complete_section(&certificate, &current(), 4) {
        CompletionOutcome::Validated(revision) => Some(revision),
        CompletionOutcome::Rejected(_) => None,
    }
    .expect("exact completion validates");
    *revision
}
#[test]
fn exact_completion_yields_one_immutable_unpublished_revision() {
    let mut stager = staged();
    let revision = finish(&mut stager);
    assert_eq!(revision.section_key(), &key("ships"));
    assert_eq!(revision.records()[0].record_id.as_str(), "record:1");
    assert_eq!(revision.records()[1].record_id.as_str(), "record:2");
    assert_eq!(revision.coverage(), CompletionCoverage::Complete);
    assert_eq!(
        revision.context().capture_window(),
        CaptureWindow::new(10, 20).expect("window is ordered")
    );
    assert!(!revision.is_published());
    assert_eq!(stager.candidate_count(), 0);
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
        stager.complete_section(&certificate, &changed, 4),
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
fn eligible_revision_becomes_stale_then_history_only_under_uncertainty() {
    let mut index = DecisionRevisionIndex::new(4).expect("blocker limit is non-zero");
    index.accept(finish(&mut staged()), 4);
    index.record_current_pointer(key("sectors"), revision(4));
    let set = match index.eligibility(&[key("ships")], 10, 10) {
        DecisionEligibility::Eligible(set) => Some(set),
        DecisionEligibility::Blocked(_) => None,
    }
    .expect("fresh exact revision is eligible");
    assert_eq!(set.revisions().len(), 1);
    assert!(set.revisions().contains_key(&key("ships")));
    assert_eq!(
        index.eligibility(&[key("ships")], 15, 10),
        DecisionEligibility::Blocked(vec![EligibilityBlocker::Stale(key("ships"))])
    );
    index.mark_scope_uncertain(&value("scope:x4", SourceScopeId::new));
    assert_eq!(index.current_count(), 0);
    assert_eq!(index.history_count(), 1);
    assert!(matches!(
        index.eligibility(&[key("ships")], 5, 10),
        DecisionEligibility::Blocked(ref blockers)
            if blockers == &[EligibilityBlocker::Uncertain(value("scope:x4", SourceScopeId::new))]
    ));
}
