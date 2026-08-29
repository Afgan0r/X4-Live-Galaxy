use crate::{CheckpointAck, CheckpointDraft, CheckpointEnvelope, CheckpointPort};
use mind_domain::{AcceptedPreemption, AcceptedProposal, PendingMindCommit, PreemptionRequest};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DeliberationCheckpointRecord {
    pub correlation_id: String,
    pub candidate_bytes: usize,
    pub policy_version: String,
    pub prompt_package_hash: String,
    pub acknowledged: CheckpointAck,
    pub compare_and_set_performed: bool,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliberationCheckpointError {
    Envelope,
    Port,
}

pub fn persist_deliberation<P>(
    port: &mut P,
    accepted: &AcceptedProposal,
    pending: &PendingMindCommit,
) -> Result<DeliberationCheckpointRecord, DeliberationCheckpointError>
where
    P: CheckpointPort,
{
    persist(port, accepted, pending, None)
}

pub fn persist_preemption<P>(
    port: &mut P,
    accepted: &AcceptedPreemption,
) -> Result<DeliberationCheckpointRecord, DeliberationCheckpointError>
where
    P: CheckpointPort,
{
    persist(
        port,
        accepted.accepted(),
        accepted.pending(),
        Some(accepted.preemption()),
    )
}

fn persist<P>(
    port: &mut P,
    accepted: &AcceptedProposal,
    pending: &PendingMindCommit,
    preemption: Option<&PreemptionRequest>,
) -> Result<DeliberationCheckpointRecord, DeliberationCheckpointError>
where
    P: CheckpointPort,
{
    if let Some(existing) = port.load() {
        return persist_existing(port, accepted, pending, &existing, preemption);
    }
    write(port, accepted, pending, 1, None, preemption)
}

fn persist_existing<P>(
    port: &mut P,
    accepted: &AcceptedProposal,
    pending: &PendingMindCommit,
    existing: &crate::CheckpointEnvelope,
    preemption: Option<&PreemptionRequest>,
) -> Result<DeliberationCheckpointRecord, DeliberationCheckpointError>
where
    P: CheckpointPort,
{
    if existing.restored_mind().ok().as_ref() == Some(pending.aggregate()) {
        if existing.causal_preemption() != preemption {
            return Err(DeliberationCheckpointError::Envelope);
        }
        return Ok(record(
            accepted,
            CheckpointAck::from_envelope(existing),
            false,
        ));
    }
    let predecessor = existing.cursor();
    write(
        port,
        accepted,
        pending,
        predecessor.sequence() + 1,
        Some(predecessor),
        preemption,
    )
}

fn write<P>(
    port: &mut P,
    accepted: &AcceptedProposal,
    pending: &PendingMindCommit,
    sequence: u64,
    predecessor: Option<crate::CheckpointCursor>,
    preemption: Option<&PreemptionRequest>,
) -> Result<DeliberationCheckpointRecord, DeliberationCheckpointError>
where
    P: CheckpointPort,
{
    let correlation_id = accepted.correlation_id();
    let draft = CheckpointDraft::new(
        accepted.snapshot_identity(),
        &format!("shadow-{correlation_id}"),
        accepted.prompt_package_hash(),
        accepted.policy_version(),
        &format!("shadow-none-{correlation_id}"),
    );
    let expected = predecessor.clone();
    let envelope = match preemption {
        Some(record) => CheckpointEnvelope::from_pending_preemption(
            sequence,
            predecessor,
            pending,
            draft,
            record.clone(),
        ),
        None => CheckpointEnvelope::from_pending_commit(sequence, predecessor, pending, draft),
    }
    .map_err(|_| DeliberationCheckpointError::Envelope)?;
    let acknowledged = port
        .compare_and_set(expected.as_ref(), envelope)
        .map_err(|_| DeliberationCheckpointError::Port)?;
    Ok(record(accepted, acknowledged, true))
}

fn record(
    accepted: &AcceptedProposal,
    acknowledged: CheckpointAck,
    compare_and_set_performed: bool,
) -> DeliberationCheckpointRecord {
    DeliberationCheckpointRecord {
        correlation_id: accepted.correlation_id().into(),
        candidate_bytes: accepted.candidate_bytes(),
        policy_version: accepted.policy_version().into(),
        prompt_package_hash: accepted.prompt_package_hash().into(),
        acknowledged,
        compare_and_set_performed,
    }
}
