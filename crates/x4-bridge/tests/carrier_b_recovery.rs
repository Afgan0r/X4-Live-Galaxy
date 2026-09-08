#![expect(
    clippy::expect_used,
    reason = "contract fixtures must fail immediately"
)]

mod carrier_b_support;

use observation_application::{LifecycleContext, LifecycleResult, ReconcileResult};
use observation_ingest::{DecisionEligibility, ReceiverDisposition};
use observation_persistence::{ObservationRepository, SqliteObservationRepository};
use x4_bridge::ProductionObservationSession;

use carrier_b_support::{
    AmbiguousRepository, FirstPublish, completion_bytes, context, current, database,
    generation_limits, input, lifecycle_limits, publication_limits, session_with, start_bytes,
};

#[test]
fn clean_restart_replays_exact_receipt_without_duplicate_publication() {
    let database = database("restart");
    publish(&database);
    publish(&database);

    let repository = SqliteObservationRepository::open(database.path(), publication_limits())
        .expect("independent repository opens");
    let key = observation_domain::SectionKey::new("runtime-clock").expect("section key");
    let current = repository
        .current(&key)
        .expect("readback succeeds")
        .expect("published revision exists");
    assert_eq!(current.receipt().ordinal, 1);
}

#[test]
fn response_loss_reconciles_the_exact_committed_attempt() {
    let database = database("response-loss");
    let repository = SqliteObservationRepository::open(database.path(), publication_limits())
        .expect("repository opens");
    let mut session = session_with(AmbiguousRepository::new(
        repository,
        FirstPublish::CommitThenAmbiguous,
    ));
    submit_start(&mut session);
    assert_eq!(
        submit_completion(&mut session),
        Ok(LifecycleResult::Disposition(
            ReceiverDisposition::AmbiguousCommit
        ))
    );
    assert_eq!(
        session.reconcile_ambiguous(3),
        Ok(LifecycleResult::Reconciled(ReconcileResult::Committed))
    );
    drop(session);
    assert_exact_recovered_revision(database.path());
}

#[test]
fn rejected_boundary_preserves_ambiguous_reconciliation() {
    let database = database("boundary-during-response-loss");
    let repository = SqliteObservationRepository::open(database.path(), publication_limits())
        .expect("repository opens");
    let mut session = session_with(AmbiguousRepository::new(
        repository,
        FirstPublish::CommitThenAmbiguous,
    ));
    submit_start(&mut session);
    assert_eq!(
        submit_completion(&mut session),
        Ok(LifecycleResult::Disposition(
            ReceiverDisposition::AmbiguousCommit
        ))
    );
    let boundary = String::from_utf8(start_bytes("runtime-clock"))
        .expect("fixture is UTF-8")
        .replacen(
            "\"producer_incarnation\":\"producer:1\"",
            "\"producer_incarnation\":\"producer:2\"",
            1,
        )
        .replacen("\"transport_epoch\":1", "\"transport_epoch\":2", 1)
        .replacen("\"section_revision\":1", "\"section_revision\":2", 1)
        .replacen(
            "\"source_epoch_status\":\"unknown\"",
            "\"source_epoch_status\":\"boundary_uncertain\"",
            1,
        )
        .replacen(
            "\"source_boundary\":\"unknown\"",
            "\"source_boundary\":\"lua_reload\"",
            1,
        )
        .into_bytes();
    assert_eq!(
        session.submit_received(
            observation_domain::TransportEpoch::new(2).expect("epoch"),
            observation_domain::BatchId::new("outer:rejected-boundary").expect("identity"),
            boundary,
            0,
            3,
        ),
        Err(x4_bridge::ProductionError::Lifecycle(
            observation_application::LifecycleError::BlockedAmbiguous
        ))
    );
    assert_eq!(
        session.reconcile_ambiguous(3),
        Ok(LifecycleResult::Reconciled(ReconcileResult::Committed))
    );
    let key = observation_domain::SectionKey::new("runtime-clock").expect("key");
    assert!(matches!(
        session.decision_eligibility(&[key], 3, 10),
        DecisionEligibility::Eligible(_)
    ));
}

fn assert_exact_recovered_revision(path: &std::path::Path) {
    let repository = SqliteObservationRepository::open(path, publication_limits())
        .expect("independent recovery readback opens");
    let snapshot = repository.current_snapshot().expect("snapshot");
    assert_eq!(snapshot.len(), 1);
    let current = &snapshot[0];
    let revision = current.revision();
    let receipt = current.receipt();
    assert_eq!(revision.section_key.as_str(), "runtime-clock");
    assert_eq!(revision.revision.get(), 1);
    assert_eq!(revision.source_scope.as_str(), "scope:x4");
    assert_eq!(
        revision.source_session.producer_incarnation().as_str(),
        "producer:1"
    );
    assert_eq!(revision.source_session.transport_epoch().get(), 1);
    assert_eq!(revision.accepted_at, 2);
    assert!(revision.records.is_empty());
    assert_eq!(receipt.section_key, revision.section_key);
    assert_eq!(receipt.revision, revision.revision);
    assert_eq!(receipt.content_digest, revision.content_digest);
    assert_eq!(receipt.ordinal, 1);
    assert_eq!(receipt.accepted_at, 2);
}

#[test]
fn proven_absent_attempt_allows_only_retained_retry() {
    let database = database("proven-absent");
    let repository = SqliteObservationRepository::open(database.path(), publication_limits())
        .expect("repository opens");
    let mut session = session_with(AmbiguousRepository::new(
        repository,
        FirstPublish::SkipThenAmbiguous,
    ));
    submit_start(&mut session);
    assert_eq!(
        submit_completion(&mut session),
        Ok(LifecycleResult::Disposition(
            ReceiverDisposition::AmbiguousCommit
        ))
    );
    assert_eq!(
        session.reconcile_ambiguous(3),
        Ok(LifecycleResult::Reconciled(
            ReconcileResult::ProvenNotCommitted
        ))
    );
    assert_eq!(
        session.retry_proven_not_committed(),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Committed))
    );
}

fn publish(database: &carrier_b_support::TempDatabase) {
    let mut session = ProductionObservationSession::open(
        database.path(),
        generation_limits(),
        publication_limits(),
        lifecycle_limits(),
        4,
    )
    .expect("production session opens");
    assert_eq!(
        session.submit(input(
            "outer:start",
            start_bytes("runtime-clock"),
            LifecycleContext::Start(context(observation_domain::SectionCoverage::KnownEmpty)),
            1,
        )),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Received))
    );
    assert_eq!(
        session.submit(input(
            "outer:complete",
            completion_bytes("runtime-clock"),
            LifecycleContext::Completion(current()),
            2,
        )),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Committed))
    );
}

fn submit_start<R: ObservationRepository>(session: &mut ProductionObservationSession<R>) {
    assert_eq!(
        session.submit(input(
            "outer:start",
            start_bytes("runtime-clock"),
            LifecycleContext::Start(context(observation_domain::SectionCoverage::KnownEmpty)),
            1,
        )),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Received))
    );
}

fn submit_completion<R: ObservationRepository>(
    session: &mut ProductionObservationSession<R>,
) -> Result<LifecycleResult, x4_bridge::ProductionError> {
    session.submit(input(
        "outer:complete",
        completion_bytes("runtime-clock"),
        LifecycleContext::Completion(current()),
        2,
    ))
}
