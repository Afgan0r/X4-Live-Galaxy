use observation_ingest::AcceptedSnapshot;
use x4_bridge::{BridgeError, CapabilityDecision, TelemetryFrame, admit_tracer_frame};

#[test]
fn protocol_contract_admits_compatible_telemetry_only_frame() {
    let fixture = include_str!("../../../tests/fixtures/tracer-observation.json");
    let decision = CapabilityDecision::negotiate("live-galaxy-observation-v2");
    let frame = TelemetryFrame::observation(fixture);

    let accepted = match admit_tracer_frame(decision, frame) {
        Ok(accepted) => accepted,
        Err(error) => {
            panic!("compatible bounded telemetry must admit the tracer fixture: {error:?}")
        }
    };

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
    let fixture = FakeX4Adapter {
        entity_id: "sector:alpha",
    }
    .produce_observation();

    let decoded = AcceptedSnapshot::from_tracer_payload(&fixture).unwrap_or_else(|error| {
        panic!("the fake adapter output remains independently decodable: {error:?}")
    });

    assert_eq!(decoded.entity_id().as_str(), "sector:alpha");
}

#[test]
fn fake_x4_adapter_contract_keeps_unavailable_and_oversized_data_explicit() {
    let unsupported = include_str!("../../../tests/fixtures/tracer-observation.json")
        .replace("\"fresh\"", "\"unsupported\"");
    let snapshot = AcceptedSnapshot::from_tracer_payload(&unsupported).unwrap_or_else(|error| {
        panic!("unsupported is an explicit observation quality: {error:?}")
    });
    let too_large = TelemetryFrame::observation("x".repeat(513));

    assert_eq!(snapshot.section_quality_name(), "unsupported");
    assert_eq!(
        admit_tracer_frame(CapabilityDecision::Compatible, too_large),
        Err(BridgeError::FrameTooLarge)
    );
}

struct FakeX4Adapter {
    entity_id: &'static str,
}

impl FakeX4Adapter {
    fn produce_observation(&self) -> String {
        format!(
            r#"{{"entity_id":"{}","observed_at_unix_millis":1725000000000,"version":1,"quality":"fresh"}}"#,
            self.entity_id
        )
    }
}
