use std::sync::atomic::{AtomicBool, AtomicU64, AtomicUsize};

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
    SecurityPolicyFailed,
    AccessProbeFailed,
    PipeCreationFailed,
    WorkerStartFailed,
    StaleGeneration,
    MessageTooLarge,
    InvalidOutputCapacity,
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
pub enum SecurityControl {
    Enforced,
    #[default]
    Missing,
}

impl From<bool> for SecurityControl {
    fn from(value: bool) -> Self {
        if value { Self::Enforced } else { Self::Missing }
    }
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct SecurityEvidence {
    pub explicit_descriptor: SecurityControl,
    pub current_logon_allowed: SecurityControl,
    pub outside_logon_denied: SecurityControl,
    pub remote_clients_rejected: SecurityControl,
}

#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct WorkerSnapshot {
    pub connected: bool,
    pub connection_generation: u64,
    pub pending_operation_owners: usize,
    pub monotonic_millis: Option<u64>,
    pub closed: bool,
}

pub(crate) struct Shared {
    pub token: HandleToken,
    pub data_busy: AtomicBool,
    pub close_requested: AtomicBool,
    pub reconnect_requested: AtomicBool,
    pub connected: AtomicBool,
    pub connection_generation: AtomicU64,
    pub closed: AtomicBool,
    pub owners: AtomicUsize,
    pub millis: AtomicU64,
    pub clock_available: AtomicBool,
}

impl Shared {
    pub fn new(token: HandleToken) -> Self {
        Self {
            token,
            data_busy: AtomicBool::new(false),
            close_requested: AtomicBool::new(false),
            reconnect_requested: AtomicBool::new(false),
            connected: AtomicBool::new(false),
            connection_generation: AtomicU64::new(0),
            closed: AtomicBool::new(false),
            owners: AtomicUsize::new(0),
            millis: AtomicU64::new(0),
            clock_available: AtomicBool::new(false),
        }
    }
}

pub(crate) fn validate(config: &TransportConfig) -> Result<(), TransportError> {
    if config.pipe_name.is_empty()
        || config.max_data_message_bytes == 0
        || config.max_control_message_bytes == 0
    {
        Err(TransportError::InvalidConfig)
    } else {
        Ok(())
    }
}
