use super::*;

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
    assert!(receiver.heavy_collection_complete());
    assert_ne!(receiver.collection_key(), "carrier_b_realtime_sample");
    assert_ne!(
        receiver.next_ship_key("faction_census", 1),
        Ok(String::from("carrier_b_realtime_sample"))
    );
}
