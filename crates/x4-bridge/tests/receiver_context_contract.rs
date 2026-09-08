mod carrier_b_support;

use observation_application::{LifecycleContext, LifecycleError, LifecycleResult};
use observation_ingest::ReceiverDisposition;
use observation_ingest::{DecisionEligibility, EligibilityBlocker};
use observation_persistence::{ObservationRepository, SqliteObservationRepository};
use x4_bridge::{ProductionError, ProductionObservationSession};

use carrier_b_support::{
    context, database, generation_limits, input, lifecycle_limits, publication_limits, start_bytes,
};

#[test]
fn sender_evidence_cannot_be_replaced_by_forged_receiver_context() {
    let database = database("receiver-context-red");
    let mut session = ProductionObservationSession::open(
        database.path(),
        generation_limits(),
        publication_limits(),
        lifecycle_limits(),
        4,
    )
    .expect("production session opens");
    let bytes = String::from_utf8(start_bytes("runtime-clock"))
        .expect("fixture is UTF-8")
        .replacen(
            "\"coverage\":\"complete\"",
            "\"coverage\":\"point_measurement\"",
            1,
        )
        .into_bytes();

    assert_eq!(
        session.submit(input(
            "outer:forged",
            bytes,
            LifecycleContext::Start(context(observation_domain::SectionCoverage::KnownEmpty,)),
            1,
        )),
        Err(ProductionError::Lifecycle(LifecycleError::ContextMismatch))
    );
}

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "boundary test keeps the ordered durable setup and replacement assertions together"
)]
fn production_bytes_derive_context_and_publish_point_measurement() {
    let database = database("receiver-context-publication");
    let mut session = ProductionObservationSession::open(
        database.path(),
        generation_limits(),
        publication_limits(),
        lifecycle_limits(),
        4,
    )
    .expect("production session opens");
    let start = String::from_utf8(start_bytes("runtime-clock"))
        .expect("fixture is UTF-8")
        .replacen(
            "\"coverage\":\"complete\"",
            "\"coverage\":\"point_measurement\"",
            1,
        )
        .into_bytes();
    let completion = String::from_utf8(carrier_b_support::completion_bytes("runtime-clock"))
        .expect("fixture is UTF-8")
        .replacen(
            "\"coverage\":\"known_empty\"",
            "\"coverage\":\"point_measurement\"",
            1,
        )
        .replacen(
            "\"coverage\":\"complete\"",
            "\"coverage\":\"point_measurement\"",
            1,
        )
        .into_bytes();
    let epoch = observation_domain::TransportEpoch::new(1).expect("epoch is non-zero");
    let start_id = observation_domain::BatchId::new("outer:start").expect("identity");
    let complete_id = observation_domain::BatchId::new("outer:complete").expect("identity");

    assert_eq!(
        session.submit_received(epoch, start_id, start, 0, 1),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Received))
    );
    let decoded_completion = observation_ingest::decode_complete_message(&completion, 8_192);
    assert!(
        decoded_completion.is_ok(),
        "completion fixture must decode before production submission: {decoded_completion:?}; {}",
        String::from_utf8_lossy(&completion)
    );
    assert_eq!(
        session.submit_received(epoch, complete_id, completion, 0, 2),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Committed))
    );
    drop(session);

    let repository =
        SqliteObservationRepository::open(database.path(), publication_limits()).expect("reopen");
    let key = observation_domain::SectionKey::new("runtime-clock").expect("section key");
    let current = repository
        .current(&key)
        .expect("read succeeds")
        .expect("revision exists");
    assert_eq!(
        current.revision().coverage,
        observation_domain::CompletionCoverage::PointMeasurement
    );
    assert_eq!(current.receipt().revision.get(), 1);
    drop(repository);

    let mut replacement = ProductionObservationSession::open(
        database.path(),
        generation_limits(),
        publication_limits(),
        lifecycle_limits(),
        4,
    )
    .expect("replacement session opens");
    let boundary_start = String::from_utf8(start_bytes("runtime-clock"))
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
            "\"source_boundary\":\"game_loaded\"",
            1,
        )
        .into_bytes();
    assert_eq!(
        replacement.submit_received(
            observation_domain::TransportEpoch::new(2).expect("epoch"),
            observation_domain::BatchId::new("outer:boundary-start").expect("identity"),
            boundary_start,
            0,
            3,
        ),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Received))
    );
    assert_eq!(
        replacement.decision_eligibility(std::slice::from_ref(&key), 3, 10),
        DecisionEligibility::Blocked(vec![EligibilityBlocker::Uncertain(
            observation_domain::SourceScopeId::new("scope:x4").expect("scope")
        )])
    );
}
