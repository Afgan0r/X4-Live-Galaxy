#![expect(clippy::expect_used, reason = "contract fixtures fail immediately")]
mod carrier_b_support;
#[path = "ship_contract/core_consistency.rs"]
mod core_consistency;
#[path = "ship_contract/faction_scope.rs"]
mod faction_scope;
#[path = "ship_contract/scoped_messages.rs"]
mod scoped_messages;
#[path = "ship_contract/replay.rs"]
mod ship_contract_replay;
mod ship_support;

use observation_application::{LifecycleLimits, LifecycleResult};
use observation_domain::{BatchId, CompleteMessage, SectionKey, TransportEpoch};
use observation_ingest::{ReceiverDisposition, decode_complete_message, encode_complete_message};
use observation_persistence::{ObservationRepository, SqliteObservationRepository};
use x4_bridge::ProductionObservationSession;

fn session(path: &std::path::Path) -> ProductionObservationSession {
    session_with_message_limit(path, 4_096)
}

fn session_with_message_limit(
    path: &std::path::Path,
    complete_message_bytes: usize,
) -> ProductionObservationSession {
    let mut receiver = ProductionObservationSession::open(
        path,
        carrier_b_support::generation_limits(),
        carrier_b_support::publication_limits(),
        LifecycleLimits::new(complete_message_bytes, 16_384, 1_000, 4)
            .expect("valid lifecycle limits"),
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

fn disposition(bytes: &[u8]) -> ReceiverDisposition {
    if matches!(
        decode_complete_message(bytes, 4096).expect("message decodes"),
        CompleteMessage::SectionCompletion(_)
    ) {
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
            Ok(LifecycleResult::Disposition(disposition(bytes)))
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
    receiver
        .select_ship_core("xenon")
        .expect("mandatory hostile observation subject");
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
            Ok(LifecycleResult::Disposition(disposition(bytes)))
        );
    }
    let repeated = ship_support::with_identities(2, "argon", ["9007199254740993"; 2]);
    for (ordinal, bytes) in repeated.iter().enumerate().take(repeated.len() - 1) {
        assert_eq!(
            submit(&mut receiver, bytes, ordinal + 5),
            Ok(LifecycleResult::Disposition(ReceiverDisposition::Received))
        );
    }
    assert_eq!(
        submit(
            &mut receiver,
            repeated.last().expect("completion"),
            repeated.len() + 4
        ),
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
