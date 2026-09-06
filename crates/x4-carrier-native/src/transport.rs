use std::time::Duration;

use crate::HandleToken;

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TransportConfig {
    pub pipe_name: String,
    pub max_data_message_bytes: usize,
    pub max_control_message_bytes: usize,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportError {
    InvalidConfig,
    Unavailable,
    StaleGeneration,
    MessageTooLarge,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum TransportSendOutcome {
    LocalHandoff,
    CapacityUnavailable,
    Rejected(TransportError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum TransportPoll {
    NoMessage,
    Message(Vec<u8>),
    Closed,
    Rejected(TransportError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloseProgress {
    Requested,
    Complete,
    StaleGeneration,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SecurityEvidence {
    pub explicit_descriptor: bool,
    pub current_logon_allowed: bool,
    pub outside_logon_denied: bool,
    pub remote_clients_rejected: bool,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorkerSnapshot {
    pub connected: bool,
    pub pending_operation_owners: usize,
    pub monotonic_millis: Option<u64>,
    pub closed: bool,
}

#[derive(Debug)]
pub struct NativeTransport {
    token: HandleToken,
}

impl NativeTransport {
    pub fn start(_config: TransportConfig, token: HandleToken) -> Result<Self, TransportError> {
        Ok(Self { token })
    }

    #[must_use]
    pub const fn security_evidence(&self) -> SecurityEvidence {
        SecurityEvidence {
            explicit_descriptor: false,
            current_logon_allowed: false,
            outside_logon_denied: false,
            remote_clients_rejected: false,
        }
    }

    pub fn try_send(&self, token: HandleToken, _bytes: &[u8]) -> TransportSendOutcome {
        if token == self.token {
            TransportSendOutcome::CapacityUnavailable
        } else {
            TransportSendOutcome::Rejected(TransportError::StaleGeneration)
        }
    }

    pub fn poll_control(&self, _token: HandleToken) -> TransportPoll {
        TransportPoll::NoMessage
    }

    pub fn request_close(&self, token: HandleToken) -> CloseProgress {
        if token == self.token {
            CloseProgress::Requested
        } else {
            CloseProgress::StaleGeneration
        }
    }

    #[must_use]
    pub const fn snapshot(&self) -> WorkerSnapshot {
        WorkerSnapshot {
            connected: false,
            pending_operation_owners: 0,
            monotonic_millis: None,
            closed: false,
        }
    }

    pub fn wait_closed_for_test(&self, _timeout: Duration) -> bool {
        false
    }
}

#[derive(Debug)]
pub struct BridgePeer;

impl BridgePeer {
    pub fn connect(_config: &TransportConfig, _timeout: Duration) -> Result<Self, TransportError> {
        Err(TransportError::Unavailable)
    }

    pub fn receive(&mut self, _capacity: usize) -> Result<Vec<u8>, TransportError> {
        Err(TransportError::Unavailable)
    }

    pub fn send_control(&mut self, _bytes: &[u8]) -> Result<(), TransportError> {
        Err(TransportError::Unavailable)
    }
}
