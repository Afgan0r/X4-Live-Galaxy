use observation_domain::{CargoObservation, ShipRecordConsistency, ShipStaleReason};

const CONTENT: &str = "profile=ship_cargo\nidentity=9007199254740993\nowner=argon\ncore_revision=7\nmember_revision=7\npolicy=2\ncapture_start=100\ncapture_end=110\nsource=x4-9.00-steam-23660954-ship-detail-source-v1\nconsistency=consistent\nconsistency_reason=none\nwares_outcome=value\nstorage_outcome=value\nreservation_policy=excluded\nware=ore|17\nstorage=solid|1000|170";

#[test]
fn detail_record_preserves_per_ship_staleness_without_weakening_the_batch() {
    let text = CONTENT
        .replace("consistency=consistent", "consistency=possibly_stale")
        .replace(
            "consistency_reason=none",
            "consistency_reason=location_changed",
        );
    let cargo = CargoObservation::from_content(&text, 2).expect("stale marker");
    assert_eq!(
        cargo.dependency.consistency,
        ShipRecordConsistency::PossiblyStale(ShipStaleReason::LocationChanged)
    );
    assert_eq!(cargo.canonical_content(), text);
}

#[test]
fn consistency_and_reason_must_form_an_exact_pair() {
    for invalid in [
        CONTENT.replace(
            "consistency_reason=none",
            "consistency_reason=location_changed",
        ),
        CONTENT.replace("consistency=consistent", "consistency=possibly_stale"),
    ] {
        assert!(CargoObservation::from_content(&invalid, 2).is_err());
    }
}
