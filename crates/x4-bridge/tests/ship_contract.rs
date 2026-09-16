#![expect(clippy::expect_used, reason = "contract fixtures fail immediately")]
mod carrier_b_support;
mod ship_support;

use observation_application::LifecycleResult;
use observation_domain::{BatchId, SectionKey, TransportEpoch};
use observation_ingest::ReceiverDisposition;
use observation_persistence::{ObservationRepository, SqliteObservationRepository};
use x4_bridge::ProductionObservationSession;

fn session(path: &std::path::Path) -> ProductionObservationSession {
    let mut receiver = ProductionObservationSession::open(
        path,
        carrier_b_support::generation_limits(),
        carrier_b_support::publication_limits(),
        carrier_b_support::lifecycle_limits(),
        4,
    )
    .expect("valid test fixture");
    receiver
        .select_ship_core("argon")
        .expect("valid test fixture");
    receiver
}

fn submit(
    session: &mut ProductionObservationSession,
    bytes: &[u8],
    ordinal: usize,
) -> Result<LifecycleResult, x4_bridge::ProductionError> {
    session.submit_received(
        TransportEpoch::new(7).expect("valid test fixture"),
        BatchId::new(format!("outer:{ordinal}")).expect("valid test fixture"),
        bytes.to_vec(),
        1,
        ordinal as u64,
    )
}

const fn disposition(ordinal: usize) -> ReceiverDisposition {
    if ordinal == 3 {
        ReceiverDisposition::Committed
    } else {
        ReceiverDisposition::Received
    }
}

#[test]
fn malformed_ship_owner_cannot_replace_durable_core() {
    let database = carrier_b_support::database("ship-owner");
    let mut receiver = session(database.path());
    let valid = ship_support::messages(1, "argon");
    for (ordinal, bytes) in valid.iter().enumerate() {
        assert_eq!(
            submit(&mut receiver, bytes, ordinal + 1),
            Ok(LifecycleResult::Disposition(if ordinal == 3 {
                ReceiverDisposition::Committed
            } else {
                ReceiverDisposition::Received
            }))
        );
    }
    let malformed = ship_support::messages(2, "argon\nowner=teladi");
    assert_eq!(
        submit(&mut receiver, &malformed[0], 5),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Received))
    );
    let rejected = submit(&mut receiver, &malformed[1], 6);
    assert!(
        rejected.is_err()
            || rejected
                == Ok(LifecycleResult::Disposition(
                    ReceiverDisposition::PermanentlyRejected
                )),
        "receiver must reject malformed ship owner before publication: {rejected:?}"
    );
    drop(receiver);
    let repository =
        SqliteObservationRepository::open(database.path(), carrier_b_support::publication_limits())
            .expect("valid test fixture");
    let current = repository
        .current(&SectionKey::new("ship_core").expect("valid test fixture"))
        .expect("valid test fixture")
        .expect("valid test fixture");
    assert_eq!(current.receipt().revision.get(), 1);
    assert_eq!(current.revision().records.len(), 2);
}

#[test]
fn ship_source_requires_receiver_selection() {
    let database = carrier_b_support::database("ship-unselected");
    let mut receiver = ProductionObservationSession::open(
        database.path(),
        carrier_b_support::generation_limits(),
        carrier_b_support::publication_limits(),
        carrier_b_support::lifecycle_limits(),
        4,
    )
    .expect("valid test fixture");
    let messages = ship_support::messages(1, "argon");
    assert!(submit(&mut receiver, &messages[0], 1).is_err());
    assert!(receiver.select_ship_core("player").is_err());
    assert!(receiver.select_ship_core("xenon").is_err());
    receiver
        .select_ship_core("teladi")
        .expect("valid test fixture");
    assert!(submit(&mut receiver, &messages[0], 1).is_err());
}

#[test]
fn repeated_ship_identity_is_atomically_rejected() {
    let database = carrier_b_support::database("ship-duplicate");
    let mut receiver = session(database.path());
    let valid = ship_support::messages(1, "argon");
    for (ordinal, bytes) in valid.iter().enumerate() {
        assert_eq!(
            submit(&mut receiver, bytes, ordinal + 1),
            Ok(LifecycleResult::Disposition(disposition(ordinal)))
        );
    }
    let repeated = ship_support::with_identities(2, "argon", ["9007199254740993"; 2]);
    for (ordinal, bytes) in repeated.iter().enumerate().take(3) {
        assert_eq!(
            submit(&mut receiver, bytes, ordinal + 5),
            Ok(LifecycleResult::Disposition(ReceiverDisposition::Received))
        );
    }
    assert_eq!(
        submit(&mut receiver, &repeated[3], 8),
        Ok(LifecycleResult::Disposition(
            ReceiverDisposition::PermanentlyRejected
        ))
    );
    drop(receiver);
    let repository =
        SqliteObservationRepository::open(database.path(), carrier_b_support::publication_limits())
            .expect("valid test fixture");
    let current = repository
        .current(&SectionKey::new("ship_core").expect("valid test fixture"))
        .expect("valid test fixture")
        .expect("valid test fixture");
    assert_eq!(current.receipt().ordinal, 1);
}

#[test]
fn reopened_ship_replay_keeps_exact_receipt_and_revision_floor() {
    let database = carrier_b_support::database("ship-replay");
    let messages = ship_support::messages(1, "argon");
    for replay in 0..2 {
        let mut receiver = session(database.path());
        for (ordinal, bytes) in messages.iter().enumerate() {
            let result = submit(&mut receiver, bytes, ordinal + 1);
            assert_eq!(
                result,
                Ok(LifecycleResult::Disposition(disposition(ordinal))),
                "replay {replay} ordinal {ordinal}"
            );
        }
        assert_eq!(
            receiver.next_revision(&SectionKey::new("ship_core").expect("valid test fixture")),
            Ok(2)
        );
    }
    let repository =
        SqliteObservationRepository::open(database.path(), carrier_b_support::publication_limits())
            .expect("valid test fixture");
    let current = repository
        .current(&SectionKey::new("ship_core").expect("valid test fixture"))
        .expect("valid test fixture")
        .expect("valid test fixture");
    assert_eq!(current.receipt().ordinal, 1);
}
