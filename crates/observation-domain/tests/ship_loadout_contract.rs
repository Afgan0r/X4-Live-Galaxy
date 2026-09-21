use observation_domain::{FieldOutcome, LoadoutObservation};

const HEADER: &str = "profile=ship_loadout\nidentity=9007199254740993\nowner=argon\ncore_revision=7\nmember_revision=7\npolicy=2\ncapture_start=100\ncapture_end=110\nsource=x4-9.00-steam-23660954-ship-detail-source-v1\nconsistency=consistent\nconsistency_reason=none\nphysical_outcome=value\nvirtual_outcome=value\nsoftware_outcome=value\nmissiles_outcome=value\nunits_outcome=value\nmissile_semantics=raw_signed_inferred_items\nunits_selector=false\nvirtual_semantics=observed_macro_thruster_inferred";
fn content() -> String {
    format!(
        "{HEADER}\nphysical=engine|1|0|engine_macro|..|\nphysical=weapon|1|9007199254740993|weapon_macro|path|group\nvirtual=thruster|1|thruster_macro\nsoftware=software_max|software_current\nmissile=missile_ware|missile_macro|-3\nunit=unit_macro|unfiltered_raw|2"
    )
}

#[test]
fn installed_state_preserves_macro_only_groups_signed_missiles_and_raw_units() {
    let text = content();
    let record = LoadoutObservation::from_content(&text, 4).expect("valid installed state");
    assert_eq!(record.canonical_content(), text);
    let FieldOutcome::Value(physical) = record.physical else {
        panic!("physical value")
    };
    assert_eq!(physical[0].component, "0");
    assert_eq!(physical[0].path, "..");
    assert_eq!(physical[0].group, "");
    let FieldOutcome::Value(missiles) = record.missiles else {
        panic!("missile value")
    };
    assert_eq!(missiles[0].amount_raw, -3);
    let FieldOutcome::Value(units) = record.units else {
        panic!("unit value")
    };
    assert_eq!(units[0].category, "unfiltered_raw");
}

#[test]
fn installed_state_rejects_wrong_selectors_numeric_ranges_and_duplicates() {
    let text = content();
    for (from, to) in [
        ("units_selector=false", "units_selector=true"),
        ("member_revision=7", "member_revision=8"),
        ("engine|1|0", "engine|0|0"),
        ("engine|1|0", "engine|1|00"),
        ("|-3", "|2147483648"),
        ("|-3", "|-2147483649"),
        ("|-3", "|-0"),
        ("unfiltered_raw|2", "unfiltered_raw|4294967296"),
    ] {
        assert!(
            LoadoutObservation::from_content(&text.replace(from, to), 4).is_err(),
            "reject {to}"
        );
    }
    assert!(LoadoutObservation::from_content(&text, 1).is_err());
    let duplicate = text.replace("physical=weapon", "physical=engine");
    assert!(LoadoutObservation::from_content(&duplicate, 4).is_err());
}

#[test]
fn missing_installed_families_are_distinct_from_empty_and_zero() {
    for state in [
        "zero",
        "empty",
        "absent",
        "unknown",
        "inaccessible",
        "unsupported",
        "stale",
    ] {
        let text = HEADER.replace("_outcome=value", &format!("_outcome={state}"));
        let record = LoadoutObservation::from_content(&text, 4).expect("explicit outcome");
        assert_eq!(record.canonical_content(), text);
    }
    assert_ne!(
        LoadoutObservation::from_content(&HEADER.replace("_outcome=value", "_outcome=absent"), 4)
            .expect("absent"),
        LoadoutObservation::from_content(&HEADER.replace("_outcome=value", "_outcome=empty"), 4)
            .expect("empty")
    );
}
