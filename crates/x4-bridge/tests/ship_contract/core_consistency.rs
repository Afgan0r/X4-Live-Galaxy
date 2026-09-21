use super::*;
use observation_domain::{ShipRecordConsistency, ShipStaleReason};

#[test]
fn one_stale_core_record_does_not_discard_its_batch() {
    let database = carrier_b_support::database("ship-stale-record");
    let mut receiver = session(database.path());
    let stale = ShipRecordConsistency::PossiblyStale(ShipStaleReason::OwnerChanged);
    let messages = ship_support::with_core_consistency(
        1,
        "argon",
        ["9007199254740993", "9007199254740995"],
        Some(stale),
    );
    for (ordinal, bytes) in messages.iter().enumerate() {
        assert_eq!(
            submit(&mut receiver, bytes, ordinal + 1),
            Ok(LifecycleResult::Disposition(disposition(bytes)))
        );
    }
    drop(receiver);
    let repository =
        SqliteObservationRepository::open(database.path(), carrier_b_support::publication_limits())
            .expect("repository reopens");
    let current = repository
        .current(&SectionKey::new("ship_core").expect("key"))
        .expect("read succeeds")
        .expect("revision exists");
    assert_eq!(current.revision().records.len(), 2);
    assert!(
        current.revision().records[0]
            .content
            .contains("consistency=possibly_stale\nconsistency_reason=owner_changed")
    );
}

#[test]
fn malformed_core_consistency_pair_is_rejected() {
    let database = carrier_b_support::database("ship-stale-malformed");
    let mut receiver = session(database.path());
    let messages = ship_support::messages(1, "argon");
    assert_eq!(
        submit(&mut receiver, &messages[0], 1),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Received))
    );
    let CompleteMessage::ImmutableBatch(mut batch) =
        decode_complete_message(&messages[1], 4096).expect("batch decodes")
    else {
        panic!("batch expected");
    };
    batch.records[0].content = batch.records[0].content.replace(
        "consistency_reason=none",
        "consistency_reason=owner_changed",
    );
    let malformed = encode_complete_message(&CompleteMessage::ImmutableBatch(batch), 4096)
        .expect("malformed semantic batch still encodes");
    let rejected = submit(&mut receiver, &malformed, 2);
    assert!(
        rejected.is_err()
            || rejected
                == Ok(LifecycleResult::Disposition(
                    ReceiverDisposition::PermanentlyRejected
                ))
    );
}
