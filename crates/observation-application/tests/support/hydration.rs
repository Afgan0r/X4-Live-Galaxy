use std::collections::BTreeMap;

use observation_application::ObservationLifecycle;
use observation_domain::{
    CompletionCoverage, ProducerIncarnationId, SectionCompletionEnvelope, SectionCoverage,
    SectionStartEnvelope, SourceScopeId,
};
use observation_ingest::{
    CompletionCurrent, CompletionOutcome, DecisionEligibility, DecisionRevisionIndex,
    FinalizationOutcome, ReceiverDisposition, ValidatedSectionRevision,
};
use observation_persistence::{
    ObservationRepository, PublicationLimits, PublishOutcome, PublishRequest,
    SqliteObservationRepository,
};

use super::flow::limits;
use super::{candidate_context, epoch, key, revision, stager};

pub fn restored_eligibility(path: &std::path::Path) -> DecisionEligibility {
    let repository = SqliteObservationRepository::open(
        path,
        PublicationLimits::new(16, 8_192).expect("publication limits are non-zero"),
    )
    .expect("SQLite repository reopens");
    let mut lifecycle = ObservationLifecycle::new(
        stager(),
        DecisionRevisionIndex::new(4).expect("blocker limit is non-zero"),
        repository,
        limits(),
    );
    assert!(lifecycle.restore_current_snapshot());
    lifecycle.decision_eligibility(&[key("alpha")], 3, 100)
}

pub fn publish(
    repository: &mut SqliteObservationRepository,
    index: &mut DecisionRevisionIndex,
    revision: ValidatedSectionRevision,
    accepted_at: u64,
) {
    let authority = index
        .prepare_publication(revision)
        .expect("publication authority prepares");
    assert!(matches!(
        repository.publish(PublishRequest::from_accepted(
            authority.clone(),
            accepted_at
        )),
        PublishOutcome::CommittedNew(_)
    ));
    assert_eq!(
        index.finalize_committed(&authority, accepted_at),
        FinalizationOutcome::Finalized
    );
}

#[expect(clippy::panic, reason = "invalid fixture completion must fail")]
pub fn validated_empty(
    section: &str,
    value: u64,
    expected: Option<observation_domain::SectionRevisionId>,
    dependencies: BTreeMap<observation_domain::SectionKey, observation_domain::SectionRevisionId>,
) -> ValidatedSectionRevision {
    let mut stager = stager();
    let base = candidate_context(SectionCoverage::KnownEmpty);
    let context = observation_ingest::CandidateContext::new(
        base.versions(),
        base.capture_window(),
        base.state(),
        dependencies.clone(),
        expected,
        true,
    );
    let start = SectionStartEnvelope {
        source_scope: SourceScopeId::new("scope:x4").expect("scope is valid"),
        producer_incarnation: ProducerIncarnationId::new("producer:1").expect("producer is valid"),
        transport_epoch: epoch(),
        section_key: key(section),
        section_revision: revision(value),
        expected_records: 0,
    };
    assert_eq!(
        stager.start_section_with_context(start.clone(), context, 1),
        ReceiverDisposition::Received
    );
    let completion = SectionCompletionEnvelope {
        source_scope: start.source_scope,
        producer_incarnation: start.producer_incarnation,
        transport_epoch: start.transport_epoch,
        section_key: start.section_key,
        section_revision: start.section_revision,
        batch_count: 0,
        record_count: 0,
        raw_bytes: 0,
        decoded_bytes: 0,
        ordered_batch_manifest_digest: [0; 32],
        canonical_content_digest: [0; 32],
        schema_version: base.versions().schema(),
        policy_version: base.versions().policy(),
        canonicalization_version: base.versions().canonicalization(),
        digest_version: base.versions().digest(),
        coverage: CompletionCoverage::KnownEmpty,
    };
    let envelope =
        observation_ingest::bind_completion_certificate(completion, &[], base.versions())
            .expect("completion certificate binds");
    let certificate = stager
        .completion_certificate(envelope)
        .expect("candidate remains staged");
    match stager.complete_section(
        &certificate,
        &CompletionCurrent::new(dependencies, expected),
        2,
    ) {
        CompletionOutcome::Validated(revision) => *revision,
        CompletionOutcome::Rejected(reason) => panic!("completion rejected: {reason:?}"),
    }
}
