use crate::{CheckpointCursor, CheckpointDraft, CheckpointError};
use mind_domain::{MindAggregate, MindCheckpointState, PendingMindCommit};
use serde::{Deserialize, Serialize};

pub const SCHEMA_VERSION: &str = "1";
pub const GAME_PROTOCOL_IDENTITY: &str = "live_galaxy.persistence.v1";
const MAX_ENVELOPE_BYTES: usize = 32_768;

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointEnvelope {
    pub(super) schema_version: String,
    pub(super) game_protocol_identity: String,
    pub(super) sequence: u64,
    pub(super) predecessor: Option<CheckpointCursor>,
    pub(super) integrity_hash: String,
    pub(super) compatibility_status: String,
    pub(super) x4_restart_required: bool,
    pub(super) payload: CheckpointPayload,
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CheckpointPayload {
    pub(super) accepted_snapshot_identity: String,
    pub(super) strategic_tick_identity: String,
    pub(super) typed_mind_commit: MindCheckpointState,
    pub(super) replay_identity: String,
    pub(super) admission_identity: String,
    pub(super) reserved_report_identity: String,
}

impl CheckpointEnvelope {
    pub fn from_pending_commit(
        sequence: u64,
        predecessor: Option<CheckpointCursor>,
        commit: &PendingMindCommit,
        draft: CheckpointDraft,
    ) -> Result<Self, CheckpointError> {
        if predecessor
            .as_ref()
            .is_some_and(|cursor| cursor.sequence >= sequence)
        {
            return Err(CheckpointError::SequenceMismatch);
        }
        if predecessor
            .as_ref()
            .is_some_and(|cursor| !crate::integrity::valid(&cursor.integrity_hash))
        {
            return Err(CheckpointError::InvalidHash);
        }
        let payload = CheckpointPayload {
            accepted_snapshot_identity: draft.snapshot,
            strategic_tick_identity: draft.tick,
            typed_mind_commit: commit.checkpoint_state(),
            replay_identity: draft.replay,
            admission_identity: draft.admission,
            reserved_report_identity: draft.report,
        };
        crate::checkpoint_validation::payload(&payload)?;
        let mut envelope = Self {
            schema_version: SCHEMA_VERSION.into(),
            game_protocol_identity: GAME_PROTOCOL_IDENTITY.into(),
            sequence,
            predecessor,
            integrity_hash: String::new(),
            compatibility_status: "compatible".into(),
            x4_restart_required: false,
            payload,
        };
        envelope.integrity_hash = envelope.calculate_integrity_hash()?;
        Ok(envelope)
    }

    pub fn encode(&self) -> Result<Vec<u8>, CheckpointError> {
        self.validate()?;
        let bytes = serde_json::to_vec(self).map_err(|_| CheckpointError::Malformed)?;
        if bytes.len() > MAX_ENVELOPE_BYTES {
            return Err(CheckpointError::Oversized);
        }
        Ok(bytes)
    }

    pub fn decode(bytes: &[u8]) -> Result<Self, CheckpointError> {
        if bytes.len() > MAX_ENVELOPE_BYTES {
            return Err(CheckpointError::Oversized);
        }
        let envelope: Self =
            serde_json::from_slice(bytes).map_err(|_| CheckpointError::Malformed)?;
        envelope.validate()?;
        Ok(envelope)
    }

    #[must_use]
    pub const fn sequence(&self) -> u64 {
        self.sequence
    }

    #[must_use]
    pub fn integrity_hash(&self) -> &str {
        &self.integrity_hash
    }

    #[must_use]
    pub fn strategic_tick_id(&self) -> &str {
        &self.payload.strategic_tick_identity
    }

    #[must_use]
    pub fn reserved_report_id(&self) -> &str {
        &self.payload.reserved_report_identity
    }

    pub fn restored_mind(&self) -> Result<MindAggregate, CheckpointError> {
        self.payload
            .typed_mind_commit
            .restore()
            .map_err(|_| CheckpointError::InvalidState)
    }

    pub(crate) fn predecessor_matches(&self, expected: Option<&CheckpointCursor>) -> bool {
        self.predecessor.as_ref() == expected
    }

    #[must_use]
    pub fn cursor(&self) -> CheckpointCursor {
        CheckpointCursor::new(self.sequence, self.integrity_hash.clone())
    }

    fn validate(&self) -> Result<(), CheckpointError> {
        if self.schema_version != SCHEMA_VERSION
            || self.game_protocol_identity != GAME_PROTOCOL_IDENTITY
            || self.compatibility_status != "compatible"
            || self.x4_restart_required
        {
            return Err(CheckpointError::InvalidIdentity);
        }
        if self
            .predecessor
            .as_ref()
            .is_some_and(|cursor| cursor.sequence >= self.sequence)
        {
            return Err(CheckpointError::SequenceMismatch);
        }
        if !crate::integrity::valid(&self.integrity_hash)
            || self
                .predecessor
                .as_ref()
                .is_some_and(|cursor| !crate::integrity::valid(&cursor.integrity_hash))
        {
            return Err(CheckpointError::InvalidHash);
        }
        crate::checkpoint_validation::payload(&self.payload)?;
        if self.integrity_hash != self.calculate_integrity_hash()? {
            return Err(CheckpointError::InvalidHash);
        }
        Ok(())
    }

    pub(super) fn calculate_integrity_hash(&self) -> Result<String, CheckpointError> {
        let binding = IntegrityBinding {
            schema_version: &self.schema_version,
            game_protocol_identity: &self.game_protocol_identity,
            sequence: self.sequence,
            predecessor: &self.predecessor,
            compatibility_status: &self.compatibility_status,
            x4_restart_required: self.x4_restart_required,
            payload: &self.payload,
        };
        let bytes = serde_json::to_vec(&binding).map_err(|_| CheckpointError::Malformed)?;
        Ok(crate::integrity::checksum(&bytes))
    }
}

#[derive(Serialize)]
struct IntegrityBinding<'a> {
    schema_version: &'a str,
    game_protocol_identity: &'a str,
    sequence: u64,
    predecessor: &'a Option<CheckpointCursor>,
    compatibility_status: &'a str,
    x4_restart_required: bool,
    payload: &'a CheckpointPayload,
}
