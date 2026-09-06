#![expect(
    clippy::expect_used,
    reason = "contract fixtures must fail immediately"
)]

mod carrier_b_support;

use observation_application::{LifecycleContext, LifecycleResult, ReconcileResult};
use observation_ingest::ReceiverDisposition;
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
