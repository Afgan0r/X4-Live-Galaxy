use super::*;

#[test]
fn faction_scoped_core_rejects_cross_faction_source_before_staging() {
    let database = carrier_b_support::database("ship-faction-scope");
    let mut receiver = ProductionObservationSession::open(
        database.path(),
        carrier_b_support::generation_limits(),
        carrier_b_support::publication_limits(),
        carrier_b_support::lifecycle_limits(),
        4,
    )
    .expect("valid test fixture");
    ship_support::admit_factions(&mut receiver, &["argon", "xenon"]);
    receiver
        .select_ship_factions(["argon", "xenon"])
        .expect("validated observation roster");
    assert_eq!(receiver.collection_key(), "ship_core:argon");
    let forged = scoped_messages::scoped_messages(1, "xenon");
    assert!(submit(&mut receiver, &forged[0], 1).is_err());
    let valid = scoped_messages::scoped_messages(1, "argon");
    assert_eq!(
        submit(&mut receiver, &valid[0], 2),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Received))
    );
}

#[test]
fn zero_ship_faction_commits_partial_observation_without_known_empty_claim() {
    let database = carrier_b_support::database("ship-zero-members");
    let mut receiver = ProductionObservationSession::open(
        database.path(),
        carrier_b_support::generation_limits(),
        carrier_b_support::publication_limits(),
        carrier_b_support::lifecycle_limits(),
        4,
    )
    .expect("valid test fixture");
    ship_support::admit_factions(&mut receiver, &["argon", "khaak"]);
    receiver
        .select_ship_factions(["argon", "khaak"])
        .expect("validated observation roster");
    let messages = scoped_messages::scoped_zero_messages(1, "argon");
    for (ordinal, bytes) in messages.iter().enumerate() {
        assert_eq!(
            submit(&mut receiver, bytes, ordinal + 1),
            Ok(LifecycleResult::Disposition(disposition(bytes)))
        );
    }
    drop(receiver);
    let repository =
        SqliteObservationRepository::open(database.path(), carrier_b_support::publication_limits())
            .expect("valid test fixture");
    let current = repository
        .current(&SectionKey::new("ship_core:argon").expect("valid test fixture"))
        .expect("valid test fixture")
        .expect("valid test fixture");
    assert!(current.revision().records.is_empty());
    assert_eq!(
        current.revision().coverage,
        observation_domain::CompletionCoverage::Partial
    );
}

#[test]
fn receiver_rejects_unvalidated_faction_authority() {
    let database = carrier_b_support::database("ship-unvalidated-faction");
    let mut receiver = ProductionObservationSession::open(
        database.path(),
        carrier_b_support::generation_limits(),
        carrier_b_support::publication_limits(),
        carrier_b_support::lifecycle_limits(),
        4,
    )
    .expect("valid test fixture");
    for faction in ["custom_mod", "visitor", "unresolved"] {
        assert!(
            receiver.select_ship_factions([faction]).is_err(),
            "receiver must not derive authority from a syntactically valid token: {faction}"
        );
    }
}

#[test]
fn receiver_requests_runtime_census_before_ship_selection() {
    let database = carrier_b_support::database("ship-faction-census-first");
    let receiver = ProductionObservationSession::open(
        database.path(),
        carrier_b_support::generation_limits(),
        carrier_b_support::publication_limits(),
        carrier_b_support::lifecycle_limits(),
        4,
    )
    .expect("valid test fixture");
    assert_eq!(receiver.collection_key(), "faction_census");
    assert!(receiver.faction_census_blocks_closure());
}
