use super::*;

#[test]
fn reopened_ship_replay_keeps_exact_receipt_and_revision_floor() {
    let database = carrier_b_support::database("ship-replay");
    let messages = ship_support::messages(1, "argon");
    for replay in 0..2 {
        let mut receiver =
            session_with_message_limit(database.path(), if replay == 0 { 4_096 } else { 2_048 });
        for (ordinal, bytes) in messages.iter().enumerate() {
            let result = submit(&mut receiver, bytes, ordinal + 1);
            assert_eq!(
                result,
                Ok(LifecycleResult::Disposition(disposition(bytes))),
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

#[test]
fn reopened_ship_replay_rejects_changed_completion_versions() {
    let database = carrier_b_support::database("ship-replay-versions");
    let messages = ship_support::messages(1, "argon");
    let mut receiver = session(database.path());
    for (ordinal, bytes) in messages.iter().enumerate() {
        assert_eq!(
            submit(&mut receiver, bytes, ordinal + 1),
            Ok(LifecycleResult::Disposition(disposition(bytes)))
        );
    }
    drop(receiver);

    for field in 0..4 {
        let mut completion = decode_complete_message(messages.last().expect("completion"), 4_096)
            .expect("completion decodes");
        let CompleteMessage::SectionCompletion(done) = &mut completion else {
            panic!("completion fixture");
        };
        match field {
            0 => {
                done.schema_version =
                    observation_domain::ObservationSchemaVersion::new(9).expect("changed schema");
            }
            1 => {
                done.policy_version =
                    observation_domain::ObservationPolicyVersion::new(9).expect("changed policy");
            }
            2 => {
                done.canonicalization_version = observation_domain::CanonicalizationVersion::new(9)
                    .expect("changed canonicalization");
            }
            _ => {
                done.digest_version =
                    observation_domain::DigestAlgorithmVersion::new(9).expect("changed digest");
            }
        }
        let bytes = encode_complete_message(&completion, 4_096).expect("completion encodes");
        let mut receiver = session(database.path());
        assert_ne!(
            submit(&mut receiver, &bytes, field + 1),
            Ok(LifecycleResult::Disposition(ReceiverDisposition::Committed)),
            "changed completion version {field} must not replay"
        );
    }
}
