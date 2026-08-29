use mind_domain::{
    admit, AdmissionDecision, CommandId, DeliberationRequest, MindAggregate, ShadowProposal,
};
use observation_ingest::{AcceptedProjection, admit_batch};
use strategic_state::{Capability, Faction, PacketLimits, derive_packets};

const FRAMES: [&str; 2] = [
    r#"{"type":"observation","scope":"ZYA","entity_id":"ZYA:military:fleet","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#,
    r#"{"type":"observation","scope":"XEN","entity_id":"XEN:threat:XEN","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"shared"}"#,
];

fn request() -> DeliberationRequest {
    let snapshot = admit_batch(AcceptedProjection::empty(), &FRAMES)
        .into_projection()
        .snapshot;
    let packets = derive_packets(&snapshot, PacketLimits::tracer()).expect("packet fixture");
    DeliberationRequest::from_packet(
        packets.packet(Faction::Zya).clone(),
        "snapshot-zya-1",
        ["ZYA:military:fleet", "XEN:threat:XEN"],
        [Capability::DefenseAndMilitaryStrategy],
        "policy-v1",
        "prompt-v1",
        2048,
        4,
    )
    .expect("frozen request")
}

fn proposal() -> Vec<u8> {
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
    .expect("proposal fixture")
}

#[test]
fn sd_002_valid_candidate_creates_pending_commit() {
    let decision = admit(&request(), &proposal());
    let AdmissionDecision::Accepted(accepted) = decision else {
        panic!("valid proposal must be accepted");
    };
    let pending = accepted.pending_commit(&MindAggregate::empty(Faction::Zya));
    assert!(pending.is_ok());
    assert_eq!(accepted.command_id(), CommandId::new("mind-zya-shadow-1"));
}

#[test]
fn sd_003_malformed_missing_and_unknown_candidates_are_rejected() {
    for candidate in [b"{".as_slice(), b"{}", b"{\"unknown\":true}"] {
        assert!(matches!(admit(&request(), candidate), AdmissionDecision::Rejected(_)));
    }
}

#[test]
fn sd_001_hidden_and_sd_013_forbidden_candidates_have_no_pending_projection() {
    let hidden = br#"{"schema_version":"schema-v1","capability":"DefenseAndMilitaryStrategy","priority":1,"horizon":"short","supporting_fact_ids":["ARG:secret"],"trade_offs":["preserve logistics"],"explanation":"hold frontier","command_id":"mind-zya-shadow-1"}"#;
    let forbidden = br#"{"schema_version":"schema-v1","capability":"EconomyAndLogistics","priority":1,"horizon":"short","supporting_fact_ids":["ZYA:military:fleet"],"trade_offs":["preserve logistics"],"explanation":"hold frontier","command_id":"mind-zya-shadow-1"}"#;
    for candidate in [hidden.as_slice(), forbidden.as_slice()] {
        let decision = admit(&request(), candidate);
        assert!(matches!(decision, AdmissionDecision::Rejected(_)));
        assert!(decision.pending_commit(&MindAggregate::empty(Faction::Zya)).is_none());
    }
}
