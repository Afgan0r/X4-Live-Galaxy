use observation_domain::SectionQuality;
use serde::Deserialize;

use crate::runtime_facts::RuntimeFacts;

#[derive(Deserialize)]
#[serde(tag = "type", rename_all = "snake_case")]
pub enum WireFrame {
    Hello(WireHello),
    Heartbeat(WireHeartbeat),
    RuntimeHealth(WireRuntimeHealth),
    Observation(WireObservation),
    CompleteMarker(WireCompleteMarker),
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum FrameHeader {
    Hello {
        protocol_major: u16,
        game_build: String,
        capabilities: Vec<String>,
        generation: u64,
    },
    Data {
        kind: &'static str,
        scope: String,
        version: u64,
        generation: u64,
        sequence: u64,
    },
}

pub fn inspect_frame(payload: &str) -> Result<FrameHeader, crate::AdmissionError> {
    if payload.len() > 2_048 {
        return Err(crate::AdmissionError::FrameTooLarge);
    }
    match serde_json::from_str(payload).map_err(|_| crate::AdmissionError::InvalidFixture)? {
        WireFrame::Hello(frame) => Ok(FrameHeader::Hello {
            protocol_major: frame.protocol_major,
            game_build: frame.game_build,
            capabilities: frame.capabilities,
            generation: frame.generation,
        }),
        WireFrame::Heartbeat(frame) => header(
            "heartbeat",
            frame.scope,
            frame.version,
            frame.generation,
            frame.sequence,
        ),
        WireFrame::RuntimeHealth(frame) => {
            (!frame.status.is_empty())
                .then_some(())
                .ok_or(crate::AdmissionError::InvalidFixture)?;
            header(
                "runtime_health",
                frame.scope,
                frame.version,
                frame.generation,
                frame.sequence,
            )
        }
        WireFrame::Observation(frame) => header(
            "observation",
            frame.scope,
            frame.version,
            frame.generation,
            frame.sequence,
        ),
        WireFrame::CompleteMarker(frame) => header(
            "complete_marker",
            frame.scope,
            frame.version,
            frame.generation,
            frame.sequence,
        ),
    }
}

fn header(
    kind: &'static str,
    scope: String,
    version: u64,
    generation: Option<u64>,
    sequence: Option<u64>,
) -> Result<FrameHeader, crate::AdmissionError> {
    let _ = observation_domain::EntityId::new(scope.clone())
        .ok_or(crate::AdmissionError::InvalidScope)?;
    let _ = observation_domain::ObservationVersion::new(version)
        .ok_or(crate::AdmissionError::InvalidVersion)?;
    let generation = generation.ok_or(crate::AdmissionError::InvalidFixture)?;
    let sequence = sequence.ok_or(crate::AdmissionError::InvalidFixture)?;
    Ok(FrameHeader::Data {
        kind,
        scope,
        version,
        generation,
        sequence,
    })
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireHello {
    pub protocol_major: u16,
    pub game_build: String,
    pub capabilities: Vec<String>,
    pub generation: u64,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireHeartbeat {
    pub scope: String,
    pub version: u64,
    pub generation: Option<u64>,
    pub sequence: Option<u64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireRuntimeHealth {
    pub scope: String,
    pub version: u64,
    pub status: String,
    pub generation: Option<u64>,
    pub sequence: Option<u64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireObservation {
    pub scope: String,
    pub entity_id: String,
    pub version: u64,
    pub quality: TracerQuality,
    pub runtime_facts: RuntimeFacts,
    pub generation: Option<u64>,
    pub sequence: Option<u64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct WireCompleteMarker {
    pub scope: String,
    pub version: u64,
    pub generation: Option<u64>,
    pub sequence: Option<u64>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
pub struct TracerObservation {
    pub entity_id: String,
    pub observed_at_unix_millis: u64,
    pub version: u64,
    pub quality: TracerQuality,
}

#[derive(Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TracerQuality {
    Fresh,
    KnownEmpty,
    Unknown,
    Partial,
    Stale,
    Unsupported,
}

impl From<TracerQuality> for SectionQuality {
    fn from(value: TracerQuality) -> Self {
        match value {
            TracerQuality::Fresh => Self::Fresh,
            TracerQuality::KnownEmpty => Self::KnownEmpty,
            TracerQuality::Unknown => Self::Unknown,
            TracerQuality::Partial => Self::Partial,
            TracerQuality::Stale => Self::Stale,
            TracerQuality::Unsupported => Self::Unsupported,
        }
    }
}
