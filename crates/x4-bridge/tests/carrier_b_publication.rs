mod carrier_b_support;

use observation_application::{LifecycleContext, LifecycleError, LifecycleResult};
use observation_ingest::ReceiverDisposition;
use observation_persistence::{ObservationRepository, SqliteObservationRepository};
use x4_bridge::{ProductionError, ProductionObservationSession};

use carrier_b_support::{
    completion_bytes, completion_bytes_at, context, current, database, generation_limits, input,
    lifecycle_limits, publication_limits, start_bytes, start_bytes_at,
};

#[test]
fn complete_section_reaches_independently_readable_sqlite() {
    let database = database("publication");
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
    drop(session);

    let repository = SqliteObservationRepository::open(database.path(), publication_limits())
        .expect("independent repository opens");
    let key = observation_domain::SectionKey::new("runtime-clock").expect("section key");
    let current = repository
        .current(&key)
        .expect("readback succeeds")
        .expect("published revision exists");
    assert_eq!(current.receipt().ordinal, 1);
    assert_eq!(current.revision().records.len(), 0);
}

#[test]
fn durable_revision_exhaustion_refuses_the_next_collection() {
    for (label, revision, expected) in [
        (
            "revision-last",
            observation_ingest::MAX_DURABLE_SECTION_REVISION - 1,
            Ok(observation_ingest::MAX_DURABLE_SECTION_REVISION),
        ),
        (
            "revision-exhausted",
            observation_ingest::MAX_DURABLE_SECTION_REVISION,
            Err(ProductionError::RevisionExhausted),
        ),
    ] {
        let database = database(label);
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
                start_bytes_at("runtime-clock", revision),
                LifecycleContext::Start(context(observation_domain::SectionCoverage::KnownEmpty,)),
                1,
            )),
            Ok(LifecycleResult::Disposition(ReceiverDisposition::Received))
        );
        assert_eq!(
            session.submit(input(
                "outer:complete",
                completion_bytes_at("runtime-clock", revision),
                LifecycleContext::Completion(current()),
                2,
            )),
            Ok(LifecycleResult::Disposition(ReceiverDisposition::Committed))
        );
        let key = observation_domain::SectionKey::new("runtime-clock").expect("section key");
        assert_eq!(session.next_revision(&key), expected);
    }
}

#[test]
fn malformed_and_context_mismatched_messages_preserve_publication_state() {
    let database = database("rejections");
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
            "outer:malformed",
            b"not-json".to_vec(),
            LifecycleContext::Batch,
            1,
        )),
        Err(ProductionError::Lifecycle(LifecycleError::DecodeRejected))
    );
    assert_eq!(
        session.submit(input(
            "outer:mismatch",
            start_bytes("runtime-clock"),
            LifecycleContext::Batch,
            2,
        )),
        Err(ProductionError::Lifecycle(LifecycleError::ContextMismatch))
    );
    let repository = SqliteObservationRepository::open(database.path(), publication_limits())
        .expect("independent repository opens");
    let key = observation_domain::SectionKey::new("runtime-clock").expect("section key");
    assert_eq!(repository.current(&key), Ok(None));
}
