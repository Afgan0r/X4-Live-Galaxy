use std::process::Command;
use x4_bridge::{HeavyShipLimits, ProductionLimits};
mod carrier_b_support;
const PROFILE: &str = include_str!("../../../config/heavy-ship-experiment.json");

#[test]
fn heavy_profile_refuses_unknown_duplicate_zero_overflow_and_policy_disagreement() {
    let valid = HeavyShipLimits::parse(PROFILE).expect("valid heavy profile");
    assert_eq!(valid.group_members, valid.bridge.max_candidate_records);
    assert_eq!(valid.bridge.max_delivery_attempts, 1);
    assert!(
        ProductionLimits::parse(PROFILE).is_none(),
        "clock semantics stay strict"
    );
    for (from, to) in [
        ("\"experimental_profile\": 1", "\"experimental_profile\": 2"),
        ("\"heavy_permits\": 1", "\"heavy_permits\": 0"),
        ("\"group_members\": 67108864", "\"group_members\": 1"),
        (
            "\"max_aggregate_records\": 134217728",
            "\"max_aggregate_records\": 127",
        ),
        (
            "\"max_aggregate_batches\": 134217728",
            "\"max_aggregate_batches\": 127",
        ),
        (
            "\"max_aggregate_work\": 134217728",
            "\"max_aggregate_work\": 131071",
        ),
        (
            "\"max_inner_records\": 1048576",
            "\"max_inner_records\": 129",
        ),
        (
            "\"max_native_calls\": 67108864",
            "\"max_native_calls\": 67108865",
        ),
        (
            "\"max_allocation_bytes\": 4194304",
            "\"max_allocation_bytes\": 67108865",
        ),
        (
            "\"callback_budget_millis\": 2",
            "\"callback_budget_millis\": 50",
        ),
        ("\"retained_revisions\": 2", "\"retained_revisions\": 1"),
        (
            "\"freshness_millis\": 30000",
            "\"freshness_millis\": 18446744073709551616",
        ),
        ("\"freshness_millis\": 30000", "\"unknown\": 30000"),
        ("\"freshness_millis\": 30000", "\"group_members\": 67108864"),
    ] {
        assert!(
            HeavyShipLimits::parse(&PROFILE.replace(from, to)).is_none(),
            "reject {to}"
        );
    }
}
#[test]
fn constructed_profile_cannot_bypass_zero_validation_at_session_admission() {
    let database = carrier_b_support::database("heavy-mutated-profile");
    let mut session = x4_bridge::ProductionObservationSession::open(
        database.path(),
        carrier_b_support::generation_limits(),
        carrier_b_support::publication_limits(),
        carrier_b_support::lifecycle_limits(),
        4,
    )
    .expect("session");
    for case in 0..3 {
        let mut profile = HeavyShipLimits::parse(PROFILE).expect("valid profile");
        match case {
            0 => profile.max_inner_records = 0,
            1 => profile.max_native_calls = 0,
            _ => profile.bridge.max_candidate_records = 0,
        }
        assert_eq!(
            session.configure_heavy(profile),
            Err(x4_bridge::ProductionError::InvalidLimits)
        );
    }
}

#[test]
fn shared_nonzero_experimental_profile_is_accepted_by_production_validator() {
    let path = std::path::Path::new(env!("CARGO_MANIFEST_DIR"))
        .join("../../config/heavy-ship-experiment.json");
    let status = Command::new(env!("CARGO_BIN_EXE_validate_limits"))
        .arg("--limits-file")
        .arg(path)
        .status()
        .expect("validator runs");
    assert!(
        status.success(),
        "assertion failed: shared_nonzero_experimental_profile_is_accepted_by_production_validator: validated heavy profile rejected"
    );
}
