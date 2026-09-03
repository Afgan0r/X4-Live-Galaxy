use observation_ingest::AcceptedSnapshot;
use x4_bridge::{
    BridgeError, CapabilityDecision, CompleteMessageSendOutcome, ConnectionState, ControlEnvelope,
    ControlPollOutcome, FacadeError, ObservationCarrierFacade, TelemetryFrame, TransportEpoch,
    admit_tracer_frame,
};

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

struct CompileOnlyCarrier;

impl ObservationCarrierFacade for CompileOnlyCarrier {
    fn connection_state(&self) -> ConnectionState {
        ConnectionState::Connected(TransportEpoch::new(2).expect("test epoch is positive"))
    }

    fn try_send_complete(&mut self, _: TransportEpoch, _: &[u8]) -> CompleteMessageSendOutcome {
        CompleteMessageSendOutcome::LocalHandoff
    }

    fn poll_control(&mut self, _: usize) -> ControlPollOutcome {
        ControlPollOutcome::Message(ControlEnvelope::Health)
    }
}

#[test]
fn compile_time_facade_is_bounded_to_observation_and_control() {
    let mut carrier = CompileOnlyCarrier;
    let epoch = TransportEpoch::new(2).expect("test epoch is positive");

    assert_eq!(
        carrier.connection_state(),
        ConnectionState::Connected(epoch)
    );
    assert_eq!(
        carrier.try_send_complete(epoch, b"complete-message"),
        CompleteMessageSendOutcome::LocalHandoff
    );
    assert_eq!(
        carrier.poll_control(1),
        ControlPollOutcome::Message(ControlEnvelope::Health)
    );
    assert_ne!(
        CompleteMessageSendOutcome::Rejected(FacadeError::StaleEpoch),
        CompleteMessageSendOutcome::CapacityUnavailable
    );
}
