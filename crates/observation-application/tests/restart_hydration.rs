mod support;

use observation_application::{LifecycleContext, LifecycleResult, ObservationLifecycle};
use observation_ingest::{DecisionRevisionIndex, ReceiverDisposition};
use observation_persistence::{
    ObservationRepository, PublicationLimits, SqliteObservationRepository,
};
use support::flow::{input, limits};
use support::versioned;
use support::{candidate_context, key, repository, revision, stager};

#[test]
fn reopen_restores_typed_eligibility_and_entity_version_fences() {
    let (database, repository) = repository("restart-hydration");
    let mut lifecycle = ObservationLifecycle::new(
        stager(),
        DecisionRevisionIndex::new(2).expect("blocker limit is non-zero"),
        repository,
        limits(),
    );
    submit_revision_two(&mut lifecycle);
    let before = lifecycle.decision_eligibility(&[key("ships")], 4, 100);
    drop(lifecycle);

    let repository = SqliteObservationRepository::open(
        database.path(),
        PublicationLimits::new(16, 8_192).expect("publication limits are non-zero"),
    )
    .expect("SQLite repository reopens");
    let current = repository
        .current(&key("ships"))
        .expect("current read succeeds")
        .expect("current revision exists");
    let hydrated = current.hydrate();
    assert_eq!(
        hydrated.context(),
        &candidate_context(observation_domain::SectionCoverage::Complete)
    );

    let mut restored = ObservationLifecycle::new(
        stager(),
        DecisionRevisionIndex::new(2).expect("blocker limit is non-zero"),
        repository,
        limits(),
    );
    assert!(restored.restore_current(&current, 3));
    assert_eq!(
        restored.decision_eligibility(&[key("ships")], 4, 100),
        before
    );
    let (start, context) = versioned::start(3, Some(revision(2)));
    assert_disposition(
        &mut restored,
        "restart:start",
        start,
        context,
        ReceiverDisposition::Received,
    );
    assert_disposition(
        &mut restored,
        "restart:lower",
        versioned::batch(3, 1, "content:v1"),
        LifecycleContext::Batch,
        ReceiverDisposition::PermanentlyRejected,
    );
}

fn submit_revision_two(lifecycle: &mut ObservationLifecycle<SqliteObservationRepository>) {
    let (start, context) = versioned::start(2, None);
    assert_disposition(
        lifecycle,
        "initial:start",
        start,
        context,
        ReceiverDisposition::Received,
    );
    assert_disposition(
        lifecycle,
        "initial:batch",
        versioned::batch(2, 2, "content:v2"),
        LifecycleContext::Batch,
        ReceiverDisposition::Received,
    );
    assert_disposition(
        lifecycle,
        "initial:complete",
        versioned::completion(2, 2, "content:v2"),
        versioned::current(None),
        ReceiverDisposition::Committed,
    );
}

fn assert_disposition(
    lifecycle: &mut ObservationLifecycle<SqliteObservationRepository>,
    identity: &str,
    bytes: Vec<u8>,
    context: LifecycleContext,
    expected: ReceiverDisposition,
) {
    assert_eq!(
        lifecycle.submit(input(identity, bytes, context, 1)),
        Ok(LifecycleResult::Disposition(expected))
    );
}
