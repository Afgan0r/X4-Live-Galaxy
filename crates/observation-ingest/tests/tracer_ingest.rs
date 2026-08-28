use observation_ingest::{AcceptedSnapshot, AdmissionError};

#[test]
fn tracer_ingest_admits_one_bounded_observation_atomically() {
    let fixture = include_str!("../../../tests/fixtures/tracer-observation.json");

    let snapshot = AcceptedSnapshot::from_tracer_payload(fixture)
        .expect("the deterministic tracer fixture must be admitted");

    assert_eq!(snapshot.entity_id().as_str(), "sector:alpha");
    assert_eq!(snapshot.version().get(), 1);
}

#[test]
fn tracer_ingest_rejects_unknown_or_oversized_input_before_snapshot_creation() {
    let unknown_field = include_str!("../../../tests/fixtures/tracer-observation.json")
        .replace("\n}", ",\n  \"unexpected\": true\n}");

    assert_eq!(
        AcceptedSnapshot::from_tracer_payload(&unknown_field),
        Err(AdmissionError::InvalidFixture)
    );
    assert_eq!(
        AcceptedSnapshot::from_tracer_payload(&"x".repeat(513)),
        Err(AdmissionError::FrameTooLarge)
    );
}
