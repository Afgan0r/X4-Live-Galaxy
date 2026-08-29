use mind_domain::{AdmissionDecision, DeliberationRequest, MindAggregate, ShadowProposal, admit};
use mind_persistence::{CheckpointPort, FakeCheckpointPort, persist_deliberation};
use observation_ingest::{AcceptedProjection, admit_batch};
use strategic_state::{Capability, Faction, PacketLimits, derive_packets};

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
    let decision = admit(&request, &prior, &bytes);
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
    assert!(port.load().is_some());
}
