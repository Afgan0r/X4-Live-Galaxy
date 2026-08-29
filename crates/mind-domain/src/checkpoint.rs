use crate::{MindAggregate, PendingMindCommit};
use serde::{Deserialize, Serialize};
pub const MIND_CHECKPOINT_SCHEMA_VERSION: u8 = 1;
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct MindCheckpointState {
    pub schema_version: u8,
    pub commit: PendingMindCommit,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum MindCheckpointError {
    UnsupportedSchema,
    Invalid,
    CapacityExceeded,
    ReplayMismatch,
}
impl MindCheckpointState {
    #[must_use]
    pub fn from_pending_commit(commit: &PendingMindCommit) -> Self {
        Self {
            schema_version: MIND_CHECKPOINT_SCHEMA_VERSION,
            commit: commit.clone(),
        }
    }
    pub fn restore(&self) -> Result<MindAggregate, MindCheckpointError> {
        crate::restore::restore(self)
    }
}
