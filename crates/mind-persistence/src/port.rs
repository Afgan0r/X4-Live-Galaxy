use crate::{CheckpointCursor, CheckpointEnvelope, GAME_PROTOCOL_IDENTITY};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CheckpointAck {
    pub cursor: CheckpointCursor,
    pub strategic_tick_identity: String,
    pub reserved_report_identity: String,
}

impl CheckpointAck {
    #[must_use]
    pub fn from_envelope(envelope: &CheckpointEnvelope) -> Self {
        Self {
            cursor: envelope.cursor(),
            strategic_tick_identity: envelope.strategic_tick_id().into(),
            reserved_report_identity: envelope.reserved_report_id().into(),
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CompatibilityStatus {
    Compatible,
    X4RestartRequired,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PortError {
    ContentCollision,
    InvalidCandidate,
    RereadMismatch,
    StalePredecessor,
}

pub trait CheckpointPort {
    fn load(&self) -> Option<CheckpointEnvelope>;
    fn compare_and_set(
        &mut self,
        expected: Option<&CheckpointCursor>,
        candidate: CheckpointEnvelope,
    ) -> Result<CheckpointAck, PortError>;
    fn reread_ack(&self, cursor: &CheckpointCursor) -> Result<CheckpointAck, PortError>;
    fn compatibility(&self, game_protocol_identity: &str) -> CompatibilityStatus;
}

#[must_use]
pub fn compatibility(game_protocol_identity: &str) -> CompatibilityStatus {
    if game_protocol_identity == GAME_PROTOCOL_IDENTITY {
        CompatibilityStatus::Compatible
    } else {
        CompatibilityStatus::X4RestartRequired
    }
}
