mod carrier_b_support;

use observation_application::LifecycleResult;
use observation_ingest::{DecisionEligibility, EligibilityBlocker, ReceiverDisposition};
use x4_bridge::ProductionObservationSession;

use carrier_b_support::{
    database, generation_limits, lifecycle_limits, publication_limits, start_bytes,
};

#[test]
fn explicit_boundaries_fence_and_restore_the_same_producer_incarnation() {
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

    let key = observation_domain::SectionKey::new("runtime-clock").expect("section key");
    assert_boundary_roundtrip(&mut session, &key, "game_loaded", 2, 2, 3);
    assert_boundary_roundtrip(&mut session, &key, "lua_reload", 3, 3, 5);
}

#[expect(
    clippy::too_many_lines,
    clippy::expect_used,
    clippy::panic,
    reason = "test helper keeps one complete explicit boundary roundtrip readable"
)]
fn assert_boundary_roundtrip(
    session: &mut ProductionObservationSession,
    key: &observation_domain::SectionKey,
    boundary: &str,
    epoch: u64,
    revision: u64,
    now: u64,
) {
    let wire = |bytes: Vec<u8>| {
        String::from_utf8(bytes)
            .expect("fixture is UTF-8")
            .replacen(
                "\"transport_epoch\":1",
                &format!("\"transport_epoch\":{epoch}"),
                1,
            )
            .replacen(
                "\"section_revision\":1",
                &format!("\"section_revision\":{revision}"),
                1,
            )
            .replacen("\"quality\":\"unknown\"", "\"quality\":\"fresh\"", 1)
            .replacen(
                "\"coverage\":\"complete\"",
                "\"coverage\":\"known_empty\"",
                1,
            )
            .replacen("\"stable_identity\":false", "\"stable_identity\":true", 1)
            .replacen(
                "\"source_boundary\":\"unknown\"",
                &format!("\"source_boundary\":\"{boundary}\""),
                1,
            )
            .into_bytes()
    };
    let epoch = observation_domain::TransportEpoch::new(epoch).expect("epoch");
    assert_eq!(
        session.submit_received(
            epoch,
            observation_domain::BatchId::new(format!("outer:{boundary}:start")).expect("identity"),
            wire(start_bytes("runtime-clock")),
            0,
            now,
        ),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Received))
    );
    assert_eq!(
        session.decision_eligibility(std::slice::from_ref(key), now, 10),
        DecisionEligibility::Blocked(vec![EligibilityBlocker::Uncertain(
            observation_domain::SourceScopeId::new("scope:x4").expect("scope")
        )])
    );
    assert_eq!(
        session.submit_received(
            epoch,
            observation_domain::BatchId::new(format!("outer:{boundary}:complete"))
                .expect("identity"),
            wire(carrier_b_support::completion_bytes("runtime-clock")),
            0,
            now + 1,
        ),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Committed))
    );
    let DecisionEligibility::Eligible(set) =
        session.decision_eligibility(std::slice::from_ref(key), now + 1, 10)
    else {
        panic!("explicit boundary baseline must restore eligibility");
    };
    assert_eq!(
        set.revisions().get(key).map(|value| value.get()),
        Some(revision)
    );
}
