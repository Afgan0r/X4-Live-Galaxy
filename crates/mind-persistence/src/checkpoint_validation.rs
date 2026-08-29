use crate::CheckpointError;
use crate::checkpoint::CheckpointPayload;

const MAX_ID_BYTES: usize = 128;
const MAX_MIND_STATE_BYTES: usize = 16_384;

pub fn payload(payload: &CheckpointPayload) -> Result<(), CheckpointError> {
    for value in [
        &payload.accepted_snapshot_identity,
        &payload.strategic_tick_identity,
        &payload.replay_identity,
        &payload.admission_identity,
        &payload.reserved_report_identity,
    ] {
        if value.is_empty() || value.len() > MAX_ID_BYTES {
            return Err(CheckpointError::InvalidIdentity);
        }
    }
    let mind_bytes =
        serde_json::to_vec(&payload.typed_mind_commit).map_err(|_| CheckpointError::Malformed)?;
    if mind_bytes.len() > MAX_MIND_STATE_BYTES {
        return Err(CheckpointError::Oversized);
    }
    payload
        .typed_mind_commit
        .restore()
        .map_err(|_| CheckpointError::InvalidState)?;
    Ok(())
}
