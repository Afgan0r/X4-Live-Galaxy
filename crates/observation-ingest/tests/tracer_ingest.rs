use observation_ingest::AcceptedSnapshot;

#[test]
fn tracer_ingest_admits_one_bounded_observation_atomically() {
    let fixture = include_str!("../../../tests/fixtures/tracer-observation.json");

    let snapshot = AcceptedSnapshot::from_tracer_payload(fixture)
        .expect("the deterministic tracer fixture must be admitted");

    assert_eq!(snapshot.entity_id().as_str(), "sector:alpha");
    assert_eq!(snapshot.version().get(), 1);
}
