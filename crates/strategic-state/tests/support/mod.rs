use observation_ingest::{
    AcceptedProjection, AdmissionError, AdmissionOutcome, ReceiptClock,
    admit_batch_with_receipt_clock,
};

struct FixtureReceiptClock;

impl ReceiptClock for FixtureReceiptClock {
    fn receipt_unix_millis(&self) -> Result<u64, AdmissionError> {
        Ok(1)
    }
}

pub fn runtime_fact_frames(frames: &[&str]) -> Vec<String> {
    frames
        .iter()
        .map(|frame| {
            let scope = string_field(frame, "scope");
            let entity_id = string_field(frame, "entity_id");
            let version = number_field(frame, "version");
            let quality = string_field(frame, "quality");
            let asset_id = format!("asset:fixture:{entity_id}");
            format!(
                r#"{{"type":"observation","scope":"{scope}","entity_id":"{entity_id}","version":{version},"quality":"{quality}","runtime_facts":{{"r":"x4_runtime","q":"{quality}","a":"available","s":[{{"i":"{entity_id}"}}],"x":[{{"i":"{asset_id}","p":"{entity_id}"}}],"c":[{{"i":"capacity:fixture:{entity_id}","p":"{asset_id}","v":1}}],"o":[{{"i":"ownership:fixture:{entity_id}","p":"{asset_id}","n":"faction:fixture"}}]}}}}"#
            )
        })
        .collect()
}

pub fn admit_runtime_fact_frames(frames: &[&str]) -> AcceptedProjection {
    let frames = runtime_fact_frames(frames);
    let input = frames.iter().map(String::as_str).collect::<Vec<_>>();
    let outcome =
        admit_batch_with_receipt_clock(AcceptedProjection::empty(), &input, &FixtureReceiptClock);
    assert!(
        matches!(outcome, AdmissionOutcome::Accepted(_)),
        "strict v2 runtime-fact fixture must be accepted before packet derivation"
    );
    outcome.into_projection()
}

#[expect(
    clippy::expect_used,
    reason = "test helper parses only static legacy fixture literals"
)]
fn string_field<'a>(frame: &'a str, name: &str) -> &'a str {
    let prefix = format!("\"{name}\":\"");
    let value = frame
        .split_once(&prefix)
        .expect("legacy strategic fixture must have a string field")
        .1;
    value
        .split_once('"')
        .expect("legacy strategic fixture string field must terminate")
        .0
}

#[expect(
    clippy::expect_used,
    reason = "test helper parses only static legacy fixture literals"
)]
fn number_field<'a>(frame: &'a str, name: &str) -> &'a str {
    let prefix = format!("\"{name}\":");
    let value = frame
        .split_once(&prefix)
        .expect("legacy strategic fixture must have a numeric field")
        .1;
    value
        .split_once(',')
        .expect("legacy strategic fixture numeric field must terminate")
        .0
}
