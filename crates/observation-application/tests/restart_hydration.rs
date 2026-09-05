mod support;

use std::collections::BTreeMap;

use observation_application::{LifecycleContext, LifecycleResult, ObservationLifecycle};
use observation_ingest::{
    DecisionEligibility, DecisionRevisionIndex, EligibilityBlocker, ReceiverDisposition,
};
use observation_persistence::{
    ObservationRepository, PublicationLimits, SqliteObservationRepository,
};
use support::flow::{input, limits};
use support::hydration::{publish, restored_eligibility, validated_empty};
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
    let before = lifecycle.decision_eligibility(&[key("ships")], 102, 100);
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
    assert_eq!(current.receipt.accepted_at, 1);
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
    assert!(restored.restore_current_snapshot(std::slice::from_ref(&current)));
    assert_eq!(
        restored.decision_eligibility(&[key("ships")], 102, 100),
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

#[test]
fn two_phase_restore_is_order_independent_and_keeps_advanced_dependency_current() {
    let (database, mut repository) = repository("restart-current-snapshot");
    let mut authority = DecisionRevisionIndex::new(4).expect("blocker limit is non-zero");
    publish(
        &mut repository,
        &mut authority,
        validated_empty("beta", 1, None, BTreeMap::new()),
        1,
    );
    publish(
        &mut repository,
        &mut authority,
        validated_empty(
            "alpha",
            1,
            None,
            BTreeMap::from([(key("beta"), revision(1))]),
        ),
        2,
    );
    publish(
        &mut repository,
        &mut authority,
        validated_empty("beta", 2, Some(revision(1)), BTreeMap::new()),
        3,
    );
    let snapshot = repository
        .current_snapshot()
        .expect("canonical current snapshot loads");
    drop(repository);

    let forward = restored_eligibility(database.path(), &snapshot);
    let mut reverse_snapshot = snapshot;
    reverse_snapshot.reverse();
    let reverse = restored_eligibility(database.path(), &reverse_snapshot);
    assert_eq!(forward, reverse);
    assert_eq!(
        forward,
        DecisionEligibility::Blocked(vec![EligibilityBlocker::DependencyMismatch(key("alpha"))])
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
