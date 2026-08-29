use observation_ingest::{AcceptedProjection, admit_batch};

const STATION_ONE: &str = r#"{"type":"observation","scope":"runtime:sectors","entity_id":"asset:station:10","version":4,"quality":"fresh","runtime_facts":{"r":"x4_runtime","q":"fresh","a":"available","s":[{"i":"sector:argon_prime"}],"x":[{"i":"asset:station:10","p":"sector:argon_prime"}],"c":[{"i":"capacity:station:10","p":"asset:station:10","v":42}],"o":[{"i":"ownership:station:10","p":"asset:station:10","n":"faction:argon"}]},"generation":1,"sequence":1}"#;

const STATION_TWO: &str = r#"{"type":"observation","scope":"runtime:sectors","entity_id":"asset:station:20","version":4,"quality":"fresh","runtime_facts":{"r":"x4_runtime","q":"fresh","a":"available","s":[{"i":"sector:second_contact"}],"x":[{"i":"asset:station:20","p":"sector:second_contact"}],"c":[{"i":"capacity:station:20","p":"asset:station:20","v":24}],"o":[{"i":"ownership:station:20","p":"asset:station:20","n":"faction:argon"}]},"generation":1,"sequence":2}"#;

const MARKER: &str = r#"{"type":"complete_marker","scope":"runtime:sectors","version":4,"generation":1,"sequence":3}"#;

#[test]
fn station_frames_replace_scope_only_with_the_completion_marker() {
    let accepted = admit_batch(
        AcceptedProjection::empty(),
        &[STATION_ONE, STATION_TWO, MARKER],
    )
    .into_projection();
    assert_eq!(
        accepted.snapshot().entity_ids(),
        ["asset:station:10", "asset:station:20"]
    );
    assert_eq!(
        accepted
            .runtime_facts("asset:station:20")
            .map(|facts| facts.capacity[0].value),
        Some(24)
    );
}
