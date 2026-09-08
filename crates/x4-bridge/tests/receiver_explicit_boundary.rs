mod carrier_b_support;

use observation_application::LifecycleResult;
use observation_ingest::{DecisionEligibility, EligibilityBlocker, ReceiverDisposition};
use x4_bridge::ProductionObservationSession;

use carrier_b_support::{
    database, generation_limits, lifecycle_limits, publication_limits, start_bytes,
};

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "boundary test keeps durable setup and exact fencing assertions together"
)]
fn explicit_game_boundary_fences_the_same_producer_incarnation() {
    let database = database("receiver-explicit-boundary");
    let mut session = ProductionObservationSession::open(
        database.path(),
        generation_limits(),
        publication_limits(),
        lifecycle_limits(),
        4,
    )
    .expect("production session opens");
    let qualify = |bytes: Vec<u8>| {
        String::from_utf8(bytes)
            .expect("fixture is UTF-8")
            .replacen("\"quality\":\"unknown\"", "\"quality\":\"fresh\"", 1)
            .replacen(
                "\"coverage\":\"complete\"",
                "\"coverage\":\"known_empty\"",
                1,
            )
            .replacen("\"stable_identity\":false", "\"stable_identity\":true", 1)
            .into_bytes()
    };
    let epoch = observation_domain::TransportEpoch::new(1).expect("epoch");
    assert_eq!(
        session.submit_received(
            epoch,
            observation_domain::BatchId::new("outer:start").expect("identity"),
            qualify(start_bytes("runtime-clock")),
            0,
            1,
        ),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Received))
    );
    assert_eq!(
        session.submit_received(
            epoch,
            observation_domain::BatchId::new("outer:complete").expect("identity"),
            qualify(carrier_b_support::completion_bytes("runtime-clock")),
            0,
            2,
        ),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Committed))
    );

    let boundary_start = String::from_utf8(start_bytes("runtime-clock"))
        .expect("fixture is UTF-8")
        .replacen("\"transport_epoch\":1", "\"transport_epoch\":2", 1)
        .replacen("\"section_revision\":1", "\"section_revision\":2", 1)
        .replacen(
            "\"source_boundary\":\"unknown\"",
            "\"source_boundary\":\"game_loaded\"",
            1,
        )
        .into_bytes();
    assert_eq!(
        session.submit_received(
            observation_domain::TransportEpoch::new(2).expect("epoch"),
            observation_domain::BatchId::new("outer:boundary").expect("identity"),
            boundary_start,
            0,
            3,
        ),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Received))
    );
    let key = observation_domain::SectionKey::new("runtime-clock").expect("section key");
    assert_eq!(
        session.decision_eligibility(std::slice::from_ref(&key), 3, 10),
        DecisionEligibility::Blocked(vec![EligibilityBlocker::Uncertain(
            observation_domain::SourceScopeId::new("scope:x4").expect("scope")
        )])
    );
}
