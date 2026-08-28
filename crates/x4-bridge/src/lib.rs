#![forbid(unsafe_code)]

use observation_ingest::{AcceptedSnapshot, AdmissionError};

const PROTOCOL_CAPABILITY: &str = "live-galaxy-observation-v1";
const MAX_TELEMETRY_FRAME_BYTES: usize = 512;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CapabilityDecision {
    Compatible,
    RestartRequired,
}

impl CapabilityDecision {
    pub fn negotiate(capability: &str) -> Self {
        if capability == PROTOCOL_CAPABILITY {
            Self::Compatible
        } else {
            Self::RestartRequired
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TelemetryFrame {
    Observation { payload: String },
}

impl TelemetryFrame {
    pub fn observation(payload: impl Into<String>) -> Self {
        Self::Observation {
            payload: payload.into(),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum BridgeError {
    RestartRequired,
    FrameTooLarge,
    Rejected(AdmissionError),
}

pub fn admit_tracer_frame(
    decision: CapabilityDecision,
    frame: TelemetryFrame,
) -> Result<AcceptedSnapshot, BridgeError> {
    if decision != CapabilityDecision::Compatible {
        return Err(BridgeError::RestartRequired);
    }

    let TelemetryFrame::Observation { payload } = frame;
    if payload.len() > MAX_TELEMETRY_FRAME_BYTES {
        return Err(BridgeError::FrameTooLarge);
    }

    AcceptedSnapshot::from_tracer_payload(&payload).map_err(BridgeError::Rejected)
}
