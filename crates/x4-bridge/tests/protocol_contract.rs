use observation_ingest::AcceptedSnapshot;
use x4_bridge::{BridgeError, CapabilityDecision, TelemetryFrame, admit_tracer_frame};

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

#[test]
fn fake_x4_adapter_contract() {
    let producer = include_str!("../../../extensions/live_galaxy/lua/live_galaxy_telemetry.lua");
    let fixture = FakeX4Adapter::produce_observation().expect("fake adapter has one observation");

    let decoded = AcceptedSnapshot::from_tracer_payload(fixture)
        .expect("the fake adapter output remains independently decodable");

    assert!(producer.contains("produce_observation"));
    assert_eq!(decoded.entity_id().as_str(), "sector:alpha");
}

#[test]
fn fake_x4_adapter_contract_keeps_unavailable_and_oversized_data_explicit() {
    let unsupported = include_str!("../../../tests/fixtures/tracer-observation.json")
        .replace("\"fresh\"", "\"unsupported\"");
    let snapshot = AcceptedSnapshot::from_tracer_payload(&unsupported)
        .expect("unsupported is an explicit observation quality");
    let too_large = TelemetryFrame::observation("x".repeat(513));

    assert_eq!(snapshot.section_quality_name(), "unsupported");
    assert_eq!(
        admit_tracer_frame(CapabilityDecision::Compatible, too_large),
        Err(BridgeError::FrameTooLarge)
    );
}

struct FakeX4Adapter;

impl FakeX4Adapter {
    fn produce_observation() -> Option<&'static str> {
        Some(include_str!(
            "../../../tests/fixtures/tracer-observation.json"
        ))
    }
}
