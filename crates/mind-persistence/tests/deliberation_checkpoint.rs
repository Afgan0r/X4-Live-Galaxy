use mind_domain::{admit, AdmissionDecision, DeliberationRequest, MindAggregate, ShadowProposal};
use mind_persistence::{persist_deliberation, FakeCheckpointPort};
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
    let packets = derive_packets(&snapshot, PacketLimits::tracer()).expect("packet fixture");
    let request = DeliberationRequest::from_packet(
        packets.packet(Faction::Zya).clone(),
        "snapshot-zya-1",
        ["ZYA:military:fleet"],
        [Capability::DefenseAndMilitaryStrategy],
        "policy-v1",
        "prompt-v1",
        2048,
        4,
    )
    .expect("request fixture");
    let bytes = serde_json::to_vec(&ShadowProposal::new(
        "schema-v1",
        Capability::DefenseAndMilitaryStrategy,
        1,
        "short",
        ["ZYA:military:fleet"],
        ["preserve logistics"],
        "hold the visible frontier",
        "mind-zya-shadow-1",
    ))
    .expect("proposal fixture");
    let AdmissionDecision::Accepted(accepted) = admit(&request, &bytes) else {
        panic!("candidate must be admitted");
    };
    let pending = accepted
        .pending_commit(&MindAggregate::empty(Faction::Zya))
        .expect("pending commit");
    let mut port = FakeCheckpointPort::new();
    let result = persist_deliberation(&mut port, &accepted, &pending);
    assert!(result.is_ok());
    assert!(port.load().is_some());
}
