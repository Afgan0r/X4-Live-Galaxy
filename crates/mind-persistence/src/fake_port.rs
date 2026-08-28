use crate::port::compatibility;
use crate::{
    CheckpointAck, CheckpointCursor, CheckpointEnvelope, CheckpointPort, CompatibilityStatus,
    PortError,
};

#[derive(Default)]
pub struct FakeCheckpointPort {
    acknowledged: Option<CheckpointEnvelope>,
}

impl FakeCheckpointPort {
    #[must_use]
    pub const fn new() -> Self {
        Self { acknowledged: None }
    }
}

impl CheckpointPort for FakeCheckpointPort {
    fn load(&self) -> Option<CheckpointEnvelope> {
        self.acknowledged.clone()
    }

    fn compare_and_set(
        &mut self,
        expected: Option<&CheckpointCursor>,
        candidate: CheckpointEnvelope,
    ) -> Result<CheckpointAck, PortError> {
        candidate
            .encode()
            .map_err(|_| PortError::InvalidCandidate)?;
        if let Some(retry_ack) = self.preflight(expected, &candidate)? {
            return Ok(retry_ack);
        }
        let acknowledgement = CheckpointAck::from_envelope(&candidate);
        self.acknowledged = Some(candidate);
        Ok(acknowledgement)
    }

    fn reread_ack(&self, cursor: &CheckpointCursor) -> Result<CheckpointAck, PortError> {
        let Some(current) = &self.acknowledged else {
            return Err(PortError::RereadMismatch);
        };
        if &current.cursor() != cursor {
            return Err(PortError::RereadMismatch);
        }
        Ok(CheckpointAck::from_envelope(current))
    }

    fn compatibility(&self, game_protocol_identity: &str) -> CompatibilityStatus {
        compatibility(game_protocol_identity)
    }
}

impl FakeCheckpointPort {
    fn preflight(
        &self,
        expected: Option<&CheckpointCursor>,
        candidate: &CheckpointEnvelope,
    ) -> Result<Option<CheckpointAck>, PortError> {
        match &self.acknowledged {
            Some(current) if current == candidate => {
                Ok(Some(CheckpointAck::from_envelope(current)))
            }
            Some(current) => {
                validate_successor(current, expected, candidate)?;
                Ok(None)
            }
            None if is_genesis(expected, candidate) => Ok(None),
            None => Err(PortError::StalePredecessor),
        }
    }
}

fn is_genesis(expected: Option<&CheckpointCursor>, candidate: &CheckpointEnvelope) -> bool {
    expected.is_none() && candidate.sequence() == 1 && candidate.predecessor_matches(None)
}

fn validate_successor(
    current: &CheckpointEnvelope,
    expected: Option<&CheckpointCursor>,
    candidate: &CheckpointEnvelope,
) -> Result<(), PortError> {
    if candidate.sequence() == current.sequence() {
        return Err(PortError::ContentCollision);
    }
    let cursor = current.cursor();
    if expected != Some(&cursor) || !candidate.predecessor_matches(expected) {
        return Err(PortError::StalePredecessor);
    }
    if candidate.sequence() != current.sequence() + 1 {
        return Err(PortError::StalePredecessor);
    }
    Ok(())
}
