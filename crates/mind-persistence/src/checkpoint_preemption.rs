use crate::{CheckpointCursor, CheckpointDraft, CheckpointEnvelope, CheckpointError};
use mind_domain::{PendingMindCommit, PreemptionRequest};

impl CheckpointEnvelope {
    pub fn from_pending_preemption(
        sequence: u64,
        predecessor: Option<CheckpointCursor>,
        commit: &PendingMindCommit,
        draft: CheckpointDraft,
        preemption: PreemptionRequest,
    ) -> Result<Self, CheckpointError> {
        Self::from_pending(sequence, predecessor, commit, draft, Some(preemption))
    }
}
