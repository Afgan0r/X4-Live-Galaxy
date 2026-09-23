#![expect(
    clippy::expect_used,
    reason = "contract fixtures fail immediately when their invariants are invalid"
)]

use observation_domain::{
    FactionObservationDisposition, FactionOrigin, FactionOriginEvidence,
    classify_observation_factions,
};

fn evidence(
    id: &str,
    origin: FactionOrigin,
    independent: Option<bool>,
    mind_candidate: bool,
) -> FactionOriginEvidence {
    FactionOriginEvidence::new(
        id,
        origin,
        independent,
        mind_candidate,
        "x4-9.00-steam-23660954-ship-detail-source-v1",
    )
    .expect("fixture evidence is valid")
}

#[test]
fn dynamic_roster_preserves_every_discovered_id_and_separates_mind_participation() {
    let inventory = [
        evidence("argon", FactionOrigin::Vanilla, Some(true), true),
        evidence("scaleplate", FactionOrigin::Vanilla, Some(true), false),
        evidence("xenon", FactionOrigin::Vanilla, Some(true), false),
        evidence("khaak", FactionOrigin::Vanilla, Some(true), false),
        evidence("player", FactionOrigin::Player, Some(false), false),
    ];
    let roster = classify_observation_factions(
        7,
        [
            "xenon",
            "custom_mod",
            "argon",
            "player",
            "khaak",
            "scaleplate",
        ],
        &inventory,
    )
    .expect("mixed census remains representable");

    assert_eq!(roster.discovery_revision(), 7);
    assert_eq!(roster.entries().len(), 6);
    for faction in ["argon", "scaleplate", "xenon", "khaak"] {
        assert_eq!(
            roster.entry(faction).unwrap().disposition(),
            FactionObservationDisposition::Included
        );
    }
    assert_eq!(
        roster.entry("player").unwrap().disposition(),
        FactionObservationDisposition::Excluded
    );
    assert_eq!(
        roster.entry("custom_mod").unwrap().disposition(),
        FactionObservationDisposition::Unknown
    );
    assert!(!roster.entry("xenon").unwrap().mind_candidate());
    assert!(!roster.entry("khaak").unwrap().mind_candidate());
    assert!(roster.has_unknown_blocker());
}

#[test]
fn uncertain_lifetime_stays_visible_and_duplicate_census_rejects() {
    let inventory = [
        evidence("visitor", FactionOrigin::Service, None, false),
        evidence("boron", FactionOrigin::Dlc, Some(true), false),
    ];
    let roster = classify_observation_factions(1, ["visitor", "boron"], &inventory)
        .expect("uncertainty is data, not a classifier failure");
    assert_eq!(
        roster.entry("visitor").unwrap().disposition(),
        FactionObservationDisposition::Unknown
    );
    assert_eq!(
        roster.entry("boron").unwrap().disposition(),
        FactionObservationDisposition::Included
    );
    assert!(classify_observation_factions(2, ["boron", "boron"], &inventory).is_err());
}

#[test]
fn exclusion_authority_and_mind_admission_follow_distinct_evidence() {
    let inventory = [
        evidence("player", FactionOrigin::Player, Some(true), true),
        evidence("modded", FactionOrigin::Modded, Some(true), true),
        evidence("service", FactionOrigin::Service, Some(false), true),
        evidence("argon", FactionOrigin::Vanilla, Some(true), true),
    ];
    let roster = classify_observation_factions(
        3,
        ["player", "modded", "service", "argon"],
        &inventory,
    )
    .expect("all evidenced origins remain classifiable");

    for id in ["player", "modded"] {
        let entry = roster.entry(id).expect("excluded origin is present");
        assert_eq!(entry.disposition(), FactionObservationDisposition::Excluded);
        assert_eq!(entry.reason(), "outside-mandatory-coverage");
        assert!(!entry.mind_candidate());
    }
    let service = roster.entry("service").expect("service origin is present");
    assert_eq!(service.disposition(), FactionObservationDisposition::Excluded);
    assert_eq!(service.reason(), "not-independent");
    assert!(!service.mind_candidate());

    let argon = roster.entry("argon").expect("included origin is present");
    assert_eq!(argon.disposition(), FactionObservationDisposition::Included);
    assert!(argon.mind_candidate());
}
