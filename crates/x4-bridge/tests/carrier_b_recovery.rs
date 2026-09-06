#![expect(clippy::expect_used, reason = "contract fixtures must fail immediately")]

mod carrier_b_support;

use observation_application::{LifecycleContext, LifecycleResult};
use observation_ingest::ReceiverDisposition;
use observation_persistence::{ObservationRepository, SqliteObservationRepository};
use x4_bridge::ProductionObservationSession;

use carrier_b_support::{
    completion_bytes, context, current, database, generation_limits, input, lifecycle_limits,
    publication_limits, start_bytes,
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
