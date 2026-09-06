#![expect(
    clippy::expect_used,
    reason = "invalid test setup must fail immediately"
)]

mod carrier_b_support;

use observation_application::{LifecycleContext, LifecycleResult};
use observation_ingest::ReceiverDisposition;
use x4_bridge::readback_revision;

#[test]
fn offline_readback_returns_exact_durable_revision_and_receipt() {
    let database = carrier_b_support::database("production-readback");
    let mut session = x4_bridge::ProductionObservationSession::open(
        database.path(),
        carrier_b_support::generation_limits(),
        carrier_b_support::publication_limits(),
        carrier_b_support::lifecycle_limits(),
        4,
    )
    .expect("production session opens");
    let context = carrier_b_support::context(observation_domain::SectionCoverage::KnownEmpty);
    let _ = session
        .submit(carrier_b_support::input(
            "batch:start:readback",
            carrier_b_support::start_bytes("economy.stations"),
            LifecycleContext::Start(context),
            100,
        ))
        .expect("start accepted");
    let result = session
        .submit(carrier_b_support::input(
            "batch:complete:readback",
            carrier_b_support::completion_bytes("economy.stations"),
            LifecycleContext::Completion(carrier_b_support::current()),
            101,
        ))
        .expect("completion accepted");
    assert_eq!(
        result,
        LifecycleResult::Disposition(ReceiverDisposition::Committed)
    );

    let output = readback_revision(database.path(), "economy.stations", 1);

    assert_eq!(
        output,
        Ok(String::from(
            "{\"section_key\":\"economy.stations\",\"section_revision\":1,\"records\":[],\"receipt\":{\"ordinal\":1,\"accepted_at\":101}}"
        ))
    );
}
