#![forbid(unsafe_code)]

use observation_ingest::{AcceptedSnapshot, AdmissionError};

const PROTOCOL_CAPABILITY: &str = "live-galaxy-observation-v1";
const GAME_FACING_BUILD: &str = "live-galaxy-x4-build-1";
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

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SessionGeneration(u64);

impl SessionGeneration {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    const fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SequenceNumber(u64);

impl SequenceNumber {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionHello {
    protocol_major: u16,
    game_build: String,
    capabilities: Vec<String>,
}

impl SessionHello {
    pub fn new<I, S>(protocol_major: u16, game_build: impl Into<String>, capabilities: I) -> Self
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        Self {
            protocol_major,
            game_build: game_build.into(),
            capabilities: capabilities.into_iter().map(Into::into).collect(),
        }
    }

    fn restart_requirement(&self, expected_protocol_major: u16) -> Option<RestartRequirement> {
        if self.protocol_major != expected_protocol_major {
            return Some(RestartRequirement::ProtocolMajorMismatch);
        }
        if self.game_build != GAME_FACING_BUILD {
            return Some(RestartRequirement::GameBuildMismatch);
        }
        if !self
            .capabilities
            .iter()
            .any(|capability| capability == PROTOCOL_CAPABILITY)
        {
            return Some(RestartRequirement::MissingRequiredCapability);
        }
        None
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RestartRequirement {
    ProtocolMajorMismatch,
    MissingRequiredCapability,
    GameBuildMismatch,
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum SessionDisposition {
    AwaitingCompatibility,
    Compatible,
    DegradedRequiresX4Restart(RestartRequirement),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SessionState {
    expected_protocol_major: u16,
    generation: SessionGeneration,
    last_sequence: Option<SequenceNumber>,
    disposition: SessionDisposition,
}

impl SessionState {
    pub const fn new(expected_protocol_major: u16) -> Self {
        Self {
            expected_protocol_major,
            generation: SessionGeneration::new(1),
            last_sequence: None,
            disposition: SessionDisposition::AwaitingCompatibility,
        }
    }

    pub fn admit_hello(&self, hello: SessionHello) -> Self {
        if !matches!(self.disposition, SessionDisposition::AwaitingCompatibility) {
            return self.clone();
        }

        let disposition = match hello.restart_requirement(self.expected_protocol_major) {
            Some(requirement) => SessionDisposition::DegradedRequiresX4Restart(requirement),
            None => SessionDisposition::Compatible,
        };
        Self {
            disposition,
            ..self.clone()
        }
    }

    pub fn reconnect(&self) -> Self {
        if matches!(self.disposition, SessionDisposition::Compatible) {
            return Self {
                generation: self.generation.next(),
                last_sequence: None,
                ..self.clone()
            };
        }
        self.clone()
    }

    pub fn accept_sequence(&self, sequence: SequenceNumber) -> Option<Self> {
        if !matches!(self.disposition, SessionDisposition::Compatible)
            || self.last_sequence.is_some_and(|last| sequence <= last)
        {
            return None;
        }

        Some(Self {
            last_sequence: Some(sequence),
            ..self.clone()
        })
    }

    pub const fn decision(&self) -> CapabilityDecision {
        match self.disposition {
            SessionDisposition::Compatible => CapabilityDecision::Compatible,
            SessionDisposition::AwaitingCompatibility
            | SessionDisposition::DegradedRequiresX4Restart(_) => {
                CapabilityDecision::RestartRequired
            }
        }
    }

    pub const fn generation(&self) -> SessionGeneration {
        self.generation
    }

    pub const fn restart_requirement(&self) -> Option<RestartRequirement> {
        match self.disposition {
            SessionDisposition::DegradedRequiresX4Restart(requirement) => Some(requirement),
            SessionDisposition::AwaitingCompatibility | SessionDisposition::Compatible => None,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameLimits {
    max_frame_bytes: usize,
    max_queue_depth: usize,
}

impl FrameLimits {
    pub const fn new(max_frame_bytes: usize, max_queue_depth: usize) -> Self {
        Self {
            max_frame_bytes,
            max_queue_depth,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackpressureOutcome {
    Accepted,
    FrameTooLarge,
    QueueSaturated,
    UnsupportedFrameKind,
    SessionNotCompatible,
    StaleSequence,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BoundedIngress {
    limits: FrameLimits,
    queued_frames: usize,
}

impl BoundedIngress {
    pub const fn new(limits: FrameLimits) -> Self {
        Self {
            limits,
            queued_frames: 0,
        }
    }

    pub fn submit(
        &self,
        session: &SessionState,
        sequence: SequenceNumber,
        kind: &str,
        payload: &str,
    ) -> (Self, BackpressureOutcome) {
        if session.decision() != CapabilityDecision::Compatible {
            return (*self, BackpressureOutcome::SessionNotCompatible);
        }
        if session.accept_sequence(sequence).is_none() {
            return (*self, BackpressureOutcome::StaleSequence);
        }
        if kind != "observation" {
            return (*self, BackpressureOutcome::UnsupportedFrameKind);
        }
        if payload.len() > self.limits.max_frame_bytes {
            return (*self, BackpressureOutcome::FrameTooLarge);
        }
        if self.queued_frames >= self.limits.max_queue_depth {
            return (*self, BackpressureOutcome::QueueSaturated);
        }

        (
            Self {
                queued_frames: self.queued_frames + 1,
                ..*self
            },
            BackpressureOutcome::Accepted,
        )
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
