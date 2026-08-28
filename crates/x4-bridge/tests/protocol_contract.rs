use x4_bridge::{admit_tracer_frame, CapabilityDecision, TelemetryFrame};

#[test]
fn protocol_contract_admits_compatible_telemetry_only_frame() {
    let fixture = include_str!("../../../tests/fixtures/tracer-observation.json");
    let decision = CapabilityDecision::negotiate("live-galaxy-observation-v1");
    let frame = TelemetryFrame::observation(fixture);

    let accepted = admit_tracer_frame(decision, frame)
        .expect("compatible bounded telemetry must admit the tracer fixture");

    assert_eq!(accepted.entity_id().as_str(), "sector:alpha");
}

#[test]
fn protocol_contract_has_no_effect_bearing_form() {
    let frame = TelemetryFrame::observation("{}");

    match frame {
        TelemetryFrame::Observation { .. } => {}
    }
}
