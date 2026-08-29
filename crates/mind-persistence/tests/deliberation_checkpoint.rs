use mind_domain::{
    AdmissionDecision, Capability, CommandId, DeliberationRequest, InitiativeCommand, InitiativeId,
    InitiativeSpec, MindAggregate, PreemptionDisposition, PreemptionRequest, ShadowProposal, admit,
    admit_preemption,
};
use mind_persistence::{
    CheckpointPort, FakeCheckpointPort, persist_deliberation, persist_preemption,
};
use observation_ingest::{AcceptedProjection, admit_batch};
use strategic_state::{Faction, PacketLimits, derive_packets};

#[test]
fn accepted_candidate_uses_one_checkpoint_compare_and_set() {
    let snapshot = admit_batch(
        AcceptedProjection::empty(),
        &[r#"{"type":"observation","scope":"ZYA","entity_id":"ZYA:military:fleet","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#],
    )
    .into_projection()
    .snapshot;
    let packets = derive_packets(&snapshot, PacketLimits::tracer());
    assert!(packets.is_ok());
    let Ok(packets) = packets else { return };
    let request = DeliberationRequest::from_packet(
        packets.packet(Faction::Zya).clone(),
        "snapshot-zya-1",
        ["ZYA:military:fleet"],
        [Capability::DefenseAndMilitaryStrategy],
        "policy-v1",
        "prompt-v1",
        2048,
        4,
    );
    assert!(request.is_ok());
    let Ok(request) = request else { return };
    let bytes = serde_json::to_vec(&ShadowProposal::new(
        "schema-v1",
        Capability::DefenseAndMilitaryStrategy,
        1,
        "short",
        ["ZYA:military:fleet"],
        ["preserve logistics"],
        "hold the visible frontier",
        "mind-zya-shadow-1",
    ));
    assert!(bytes.is_ok());
    let Ok(bytes) = bytes else { return };
    let prior = MindAggregate::empty(Faction::Zya);
    let decision = admit(&request, &prior, request.snapshot_identity(), &bytes);
    assert!(matches!(decision, AdmissionDecision::Accepted(_)));
    let AdmissionDecision::Accepted(accepted) = decision else {
        return;
    };
    let pending = accepted.pending_commit(&prior);
    assert!(pending.is_ok());
    let Ok(pending) = pending else { return };
    let mut port = FakeCheckpointPort::new();
    let result = persist_deliberation(&mut port, &accepted, &pending);
    assert!(result.is_ok());
    let Ok(result) = result else { return };
    assert!(result.compare_and_set_performed);
    assert!(port.load().is_some());
    let retry = persist_deliberation(&mut port, &accepted, &pending);
    assert!(retry.is_ok());
    let Ok(retry) = retry else { return };
    assert!(!retry.compare_and_set_performed);
    assert_eq!(retry.acknowledged.cursor, result.acknowledged.cursor);
}

#[test]
fn accepted_preemption_persists_full_causal_record_and_replays_exactly() {
    let request = request();
    assert!(request.is_ok());
    let Ok(request) = request else { return };
    let bytes = candidate_bytes();
    assert!(bytes.is_ok());
    let Ok(bytes) = bytes else { return };
    let prior = prior_with_active_initiative();
    assert!(prior.is_ok());
    let Ok(prior) = prior else { return };
    let Some(active) = prior.active_initiative(Capability::DefenseAndMilitaryStrategy) else {
        return;
    };
    let preemption = PreemptionRequest::new(
        CommandId::new("mind-zya-shadow-1"),
        "visible xen threat",
        active.clone(),
        PreemptionDisposition::Cancelled,
        initiative("initiative-b"),
        mind_domain::ExecutiveDecision::Approve,
        "replace the stale defense initiative",
    );
    assert!(preemption.is_ok());
    let Ok(preemption) = preemption else { return };
    let accepted = admit_preemption(
        &request,
        &prior,
        request.snapshot_identity(),
        &bytes,
        preemption.clone(),
    );
    assert!(accepted.is_ok());
    let Ok(accepted) = accepted else { return };

    let mut port = FakeCheckpointPort::new();
    let persisted = persist_preemption(&mut port, &accepted);
    assert!(persisted.is_ok());
    let Ok(persisted) = persisted else { return };
    assert!(persisted.compare_and_set_performed);
    let Some(envelope) = port.load() else { return };
    assert_eq!(envelope.causal_preemption(), Some(&preemption));
    let encoded = envelope.encode();
    assert!(encoded.is_ok());
    let Ok(encoded) = encoded else { return };
    let replayed = mind_persistence::CheckpointEnvelope::decode(&encoded);
    assert!(replayed.is_ok());
    let Ok(replayed) = replayed else { return };
    assert_eq!(replayed.causal_preemption(), Some(&preemption));
    assert_eq!(
        replayed.restored_mind(),
        Ok(accepted.pending().aggregate().clone())
    );

    let retry = persist_preemption(&mut port, &accepted);
    assert!(retry.is_ok());
    assert!(matches!(retry, Ok(record) if !record.compare_and_set_performed));
}

#[path = "deliberation_checkpoint/stale_preemption.rs"]
mod stale_preemption;

fn request() -> Result<DeliberationRequest, mind_domain::RequestError> {
    let snapshot = admit_batch(
        AcceptedProjection::empty(),
        &[r#"{"type":"observation","scope":"ZYA","entity_id":"ZYA:military:fleet","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#],
    )
    .into_projection()
    .snapshot;
    let packets = derive_packets(&snapshot, PacketLimits::tracer())
        .map_err(|_| mind_domain::RequestError::Invalid)?;
    DeliberationRequest::from_packet(
        packets.packet(Faction::Zya).clone(),
        "snapshot-zya-1",
        ["ZYA:military:fleet"],
        [Capability::DefenseAndMilitaryStrategy],
        "policy-v1",
        "prompt-v1",
        2048,
        4,
    )
}

fn candidate_bytes() -> Result<Vec<u8>, serde_json::Error> {
    serde_json::to_vec(&ShadowProposal::new(
        "schema-v1",
        Capability::DefenseAndMilitaryStrategy,
        1,
        "short",
        ["ZYA:military:fleet"],
        ["preserve logistics"],
        "hold the visible frontier",
        "mind-zya-shadow-1",
    ))
}

fn prior_with_active_initiative() -> Result<MindAggregate, mind_domain::MindError> {
    MindAggregate::empty(Faction::Zya)
        .apply_initiative(InitiativeCommand::accept(
            CommandId::new("initiative-a-command"),
            initiative("initiative-a"),
        ))
        .map(|commit| commit.aggregate().clone())
}

fn initiative(id: &str) -> InitiativeSpec {
    InitiativeSpec::new(
        InitiativeId::new(id),
        Capability::DefenseAndMilitaryStrategy,
        "defend frontier",
        "ZYA:military:fleet",
        1,
    )
}
