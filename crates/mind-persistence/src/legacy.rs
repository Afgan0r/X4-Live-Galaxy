use crate::checkpoint::CheckpointPayload;
use crate::{CheckpointEnvelope, CheckpointError, GAME_PROTOCOL_IDENTITY, SCHEMA_VERSION};
use mind_domain::MindCheckpointState;
use serde::Deserialize;

const MAX_LEGACY_BYTES: usize = 32_768;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct LegacyV0 {
    sequence: u64,
    snapshot: String,
    tick: String,
    mind: MindCheckpointState,
    replay: String,
    admission: String,
    report: String,
}

pub fn decode_and_convert(bytes: &[u8]) -> Result<CheckpointEnvelope, CheckpointError> {
    if bytes.len() > MAX_LEGACY_BYTES {
        return Err(CheckpointError::Oversized);
    }
    let legacy: LegacyV0 = serde_json::from_slice(bytes).map_err(|_| CheckpointError::Malformed)?;
    let payload = CheckpointPayload {
        accepted_snapshot_identity: legacy.snapshot,
        strategic_tick_identity: legacy.tick,
        typed_mind_commit: legacy.mind,
        replay_identity: legacy.replay,
        admission_identity: legacy.admission,
        reserved_report_identity: legacy.report,
        causal_preemption: None,
    };
    crate::checkpoint_validation::payload(&payload)?;
    let mut envelope = CheckpointEnvelope {
        schema_version: SCHEMA_VERSION.into(),
        game_protocol_identity: GAME_PROTOCOL_IDENTITY.into(),
        sequence: legacy.sequence,
        predecessor: None,
        integrity_hash: String::new(),
        compatibility_status: "compatible".into(),
        x4_restart_required: false,
        payload,
    };
    envelope.integrity_hash = envelope.calculate_integrity_hash()?;
    Ok(envelope)
}
