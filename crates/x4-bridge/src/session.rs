use crate::{CapabilityDecision, RestartRequirement, SessionHello};

#[must_use]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SessionGeneration(u64);

impl SessionGeneration {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }

    const fn next(self) -> Self {
        Self(self.0 + 1)
    }
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub struct SequenceNumber(u64);

impl SequenceNumber {
    pub const fn new(value: u64) -> Self {
        Self(value)
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
enum SessionDisposition {
    AwaitingCompatibility,
    Compatible,
    DegradedRequiresX4Restart(RestartRequirement),
}

#[must_use]
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

    #[must_use]
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

    #[must_use]
    pub const fn restart_requirement(&self) -> Option<RestartRequirement> {
        match self.disposition {
            SessionDisposition::DegradedRequiresX4Restart(requirement) => Some(requirement),
            SessionDisposition::AwaitingCompatibility | SessionDisposition::Compatible => None,
        }
    }
}
