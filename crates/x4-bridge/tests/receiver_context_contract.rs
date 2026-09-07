mod carrier_b_support;

use observation_application::{LifecycleContext, LifecycleError, LifecycleResult};
use observation_ingest::ReceiverDisposition;
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
}
