use observation_domain::{
    SectionAvailability, SectionCoverage, SectionQuality, SenderEvidence, SourceBoundary,
    SourceConsistency, SourceEpochStatus,
};

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ProducerLimits {
    pub data_message_bytes: usize,
    pub control_message_bytes: usize,
    pub max_records: usize,
    pub max_raw_bytes: usize,
    pub max_batches: usize,
    pub max_work: usize,
    pub max_retry_age_millis: u64,
}

impl ProducerLimits {
    #[must_use]
    pub const fn bring_up() -> Self {
        Self {
            data_message_bytes: 2_048,
            control_message_bytes: 512,
            max_records: 1,
            max_raw_bytes: 96,
            max_batches: 1,
            max_work: 1,
            max_retry_age_millis: 5_000,
        }
    }

    #[must_use]
    pub const fn valid(self) -> bool {
        self.data_message_bytes > 0
            && self.control_message_bytes > 0
            && self.max_records > 0
            && self.max_raw_bytes > 0
            && self.max_batches > 0
            && self.max_work > 0
            && self.max_retry_age_millis > 0
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ProducerSource {
    pub session_id: String,
    pub producer_incarnation: String,
    pub transport_epoch: u64,
    pub source_scope: String,
    pub source_epoch_status: SourceEpochStatus,
    pub source_boundary: SourceBoundary,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct SectionEvidence {
    pub source_scope: String,
    pub sender: SenderEvidence,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct SectionFinishEvidence {
    pub capture_end_millis: u64,
    pub succeeded: bool,
    pub quality: SectionQuality,
    pub availability: SectionAvailability,
    pub coverage: SectionCoverage,
    pub consistency: SourceConsistency,
    pub stable_identity: bool,
}

impl SectionEvidence {
    pub fn point_measurement(source_scope: impl Into<String>) -> Self {
        let mut sender = SenderEvidence::legacy_default();
        sender.section_state = observation_domain::SectionState::new(
            observation_domain::SectionFreshness::Fresh,
            SectionCoverage::PointMeasurement,
        );
        Self {
            source_scope: source_scope.into(),
            sender,
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct TypedFact {
    pub entity_id: String,
    pub observation_version: u64,
    pub getter: String,
    pub semantics: String,
    pub raw_value: String,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProducerState {
    AwaitingCompatibility,
    Ready,
    SectionReserved,
    Collecting,
    PendingStart,
    PendingBatch,
    PendingCompletion,
    PausedAfterFailure,
    Incompatible,
    Closed,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProducerOutcome {
    Accepted,
    Progress,
    CapacityUnavailable,
    NoControl,
    Received,
    Committed,
    PermanentlyRejected,
    Ambiguous,
    PausedAfterFailure,
    Disconnected,
    RestartRequired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProducerFeedback {
    CapacityUnavailable,
    Received,
    Committed,
    PermanentlyRejected,
    Ambiguous,
    StaleEpoch,
    TimedOutOrSuperseded,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ProducerError {
    InvalidInput,
    InvalidTransition,
    DataLimit,
    ControlLimit,
    StaleEpoch,
    Incompatible,
    ClockUnavailable,
}
