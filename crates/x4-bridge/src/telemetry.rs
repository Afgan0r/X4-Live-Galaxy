use observation_ingest::{AcceptedSnapshot, AdmissionError};

use crate::CapabilityDecision;

const MAX_TELEMETRY_FRAME_BYTES: usize = 512;

#[must_use]
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
