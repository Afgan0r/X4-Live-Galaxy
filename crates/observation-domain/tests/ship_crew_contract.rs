use observation_domain::{CrewObservation, FieldOutcome};
const HEADER: &str = "profile=ship_crew\nidentity=9007199254740993\nowner=argon\ncore_revision=7\nmember_revision=7\npolicy=2\ncapture_start=100\ncapture_end=110\nsource=x4-9.00-steam-23660954-ship-detail-source-v1\ncapacity_outcome=value\nincludepilot=true\nincludearriving=true\nrole_coverage=observed_count_fill_only\nroles_outcome=value\ncapacity_people=60";

#[test]
fn aggregate_crew_roundtrip_preserves_signed_raw_tiers_and_every_role() {
    let text = format!(
        "{HEADER}\nrole=marine|7|2|true\ntier=marine|Novice|-25|3\ntier=marine|Veteran|80|4\nrole=pilot|0|0|false\nrole=prisoner|2|0|false"
    );
    let crew = CrewObservation::from_content(&text, 4).expect("valid fixture");
    assert_eq!(crew.canonical_content(), text);
    assert_eq!(crew.capacity, FieldOutcome::Value(60));
    let FieldOutcome::Value(roles) = crew.roles else {
        panic!("observed roles");
    };
    assert_eq!(roles.len(), 3);
    assert_eq!(roles[0].tiers[0].skill_lower_threshold, -25);
    assert_eq!(roles[0].tiers[1].amount_people, 4);
    assert_eq!(roles[1].id, "pilot");
    assert_eq!(roles[1].amount_people, 0);
    assert_eq!(roles[2].id, "prisoner");
    assert!(!roles[2].canhire);
}
#[test]
fn aggregate_crew_rejects_bad_selectors_bounds_and_signed_conversion() {
    let text = format!("{HEADER}\nrole=marine|7|1|true\ntier=marine|Novice|-25|7");
    for (from, to) in [
        ("includepilot=true", "includepilot=false"),
        ("includearriving=true", "includearriving=false"),
        ("-25|7", "2147483648|7"),
        ("-25|7", "-2147483649|7"),
        ("-25|7", "-0|7"),
        ("marine|7|1", "marine|4294967296|1"),
        ("marine|7|1", "marine|7|5"),
        ("tier=marine", "tier=pilot"),
        ("member_revision=7", "member_revision=8"),
    ] {
        assert!(
            CrewObservation::from_content(&text.replace(from, to), 4).is_err(),
            "reject {to}"
        );
    }
    assert!(CrewObservation::from_content(&text, 0).is_err());
}
#[test]
fn missing_roles_and_pilot_are_not_fabricated_as_zero() {
    for state in [
        "zero",
        "empty",
        "absent",
        "unknown",
        "inaccessible",
        "unsupported",
        "stale",
    ] {
        let text = HEADER.replace("roles_outcome=value", &format!("roles_outcome={state}"));
        let crew = CrewObservation::from_content(&text, 4).expect("explicit outcome fixture");
        assert_eq!(crew.canonical_content(), text);
    }
    let missing = HEADER.replace("roles_outcome=value", "roles_outcome=absent");
    let empty = HEADER.replace("roles_outcome=value", "roles_outcome=empty");
    assert_ne!(
        CrewObservation::from_content(&missing, 4).expect("fixture"),
        CrewObservation::from_content(&empty, 4).expect("fixture")
    );
}
