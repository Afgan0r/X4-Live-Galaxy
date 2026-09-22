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
    let inventory = vec![
        observation_domain::FactionOriginEvidence::new(
            "argon",
            observation_domain::FactionOrigin::Vanilla,
            Some(true),
            true,
            "base",
        )
        .expect("included"),
        observation_domain::FactionOriginEvidence::new(
            "visitor",
            observation_domain::FactionOrigin::Service,
            Some(false),
            false,
            "base:hidden",
        )
        .expect("excluded"),
    ];
    receiver
        .accept_faction_census(1, ["argon", "visitor", "unresolved"], &inventory)
        .expect("valid mixed roster");
    assert!(receiver.select_ship_factions(["argon"]).is_ok());
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
    assert_eq!(receiver.collection_key(), "carrier_b_realtime_sample");
    assert!(receiver.faction_census_blocks_closure());
}

#[test]
fn one_faction_census_keeps_scoped_identity_and_starts_with_core() {
    let database = carrier_b_support::database("ship-single-faction-scope");
    let mut receiver = ProductionObservationSession::open(
        database.path(),
        carrier_b_support::generation_limits(),
        carrier_b_support::publication_limits(),
        carrier_b_support::lifecycle_limits(),
        4,
    )
    .expect("valid test fixture");
    ship_support::admit_factions(&mut receiver, &["argon"]);
    receiver
        .configure_heavy(
            x4_bridge::HeavyShipLimits::parse(include_str!(
                "../../../../config/heavy-ship-experiment.json"
            ))
            .expect("heavy profile"),
        )
        .expect("configured");
    assert_eq!(receiver.collection_key(), "ship_core:argon");
    assert_eq!(
        receiver.next_ship_key("faction_census", 1),
        Ok(String::from("ship_core:argon"))
    );
}

#[test]
fn reconnect_invalidates_roster_and_requires_a_fresh_census() {
    let database = carrier_b_support::database("ship-roster-reconnect");
    let mut receiver = ProductionObservationSession::open(
        database.path(),
        carrier_b_support::generation_limits(),
        carrier_b_support::publication_limits(),
        carrier_b_support::lifecycle_limits(),
        4,
    )
    .expect("valid test fixture");
    ship_support::admit_factions(&mut receiver, &["argon"]);
    assert_eq!(receiver.collection_key(), "ship_core:argon");
    receiver.invalidate_source_scope(
        &observation_domain::SourceScopeId::new("x4:faction:argon:ships").expect("scope"),
    );
    assert_eq!(receiver.collection_key(), "faction_census");
    assert!(receiver.select_ship_factions(["argon"]).is_err());
}

#[test]
fn all_excluded_census_never_schedules_the_legacy_collection() {
    let database = carrier_b_support::database("ship-all-excluded");
    let mut receiver = ProductionObservationSession::open(
        database.path(),
        carrier_b_support::generation_limits(),
        carrier_b_support::publication_limits(),
        carrier_b_support::lifecycle_limits(),
        4,
    )
    .expect("valid test fixture");
    let inventory = [observation_domain::FactionOriginEvidence::new(
        "visitor",
        observation_domain::FactionOrigin::Service,
        Some(false),
        false,
        "base:hidden",
    )
    .expect("excluded")];
    receiver
        .configure_heavy(
            x4_bridge::HeavyShipLimits::parse(include_str!(
                "../../../../config/heavy-ship-experiment.json"
            ))
            .expect("heavy profile"),
        )
        .expect("configured");
    receiver
        .accept_faction_census(1, ["visitor"], &inventory)
        .expect("valid excluded roster");
    assert!(!receiver.faction_census_blocks_closure());
    assert_ne!(receiver.collection_key(), "carrier_b_realtime_sample");
    assert_ne!(
        receiver.next_ship_key("faction_census", 1),
        Ok(String::from("carrier_b_realtime_sample"))
    );
}
