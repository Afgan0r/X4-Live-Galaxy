use mind_domain::{CommandId, MindAggregate, MindCommand, transition};
use mind_persistence::{CheckpointDraft, CheckpointEnvelope, CheckpointError};
use observation_ingest::{AcceptedProjection, admit_batch};
use strategic_state::{Faction, PacketLimits, derive_packets};

const FRAMES: [&str; 4] = [
    r#"{"type":"observation","scope":"ZYA","entity_id":"ZYA:military:fleet","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#,
    r#"{"type":"observation","scope":"ARG","entity_id":"ARG:economy:ore","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#,
    r#"{"type":"observation","scope":"ZYA","entity_id":"ZYA:territorial:station","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#,
    r#"{"type":"observation","scope":"XEN","entity_id":"XEN:threat:XEN","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"shared"}"#,
];

fn committed_mind() -> mind_domain::PendingMindCommit {
    let snapshot = admit_batch(AcceptedProjection::empty(), &FRAMES)
        .into_projection()
        .snapshot;
    let packets = derive_packets(&snapshot, PacketLimits::tracer()).unwrap();
    let packet = packets.packet(Faction::Zya);
    transition(
        &MindAggregate::empty(Faction::Zya),
        MindCommand::from_packet(packet, CommandId::new("mind-zya-1")),
    )
    .unwrap()
}

fn draft() -> CheckpointDraft {
    CheckpointDraft::new(
        "snapshot-zya-1",
        "tick-zya-1",
        "replay-zya-1",
        "admission-zya-1",
        "report-zya-1",
    )
}

#[test]
fn encodes_a_complete_deterministic_committed_transition() {
    let commit = committed_mind();
    let first = CheckpointEnvelope::from_pending_commit(1, None, &commit, draft()).unwrap();
    let second = CheckpointEnvelope::from_pending_commit(1, None, &commit, draft()).unwrap();

    assert_eq!(first.encode().unwrap(), second.encode().unwrap());
    assert_eq!(first.integrity_hash(), second.integrity_hash());
    assert_eq!(first.sequence(), 1);
    assert_eq!(first.strategic_tick_id(), "tick-zya-1");
    assert_eq!(CheckpointEnvelope::decode(&first.encode().unwrap()), Ok(first));
}

#[test]
fn rejects_tampered_partial_and_oversized_checkpoint_records() {
    let commit = committed_mind();
    let envelope = CheckpointEnvelope::from_pending_commit(1, None, &commit, draft()).unwrap();
    let mut encoded = envelope.encode().unwrap();
    encoded[10] ^= 1;
    assert_eq!(CheckpointEnvelope::decode(&encoded), Err(CheckpointError::InvalidHash));
    assert!(matches!(CheckpointEnvelope::decode(b"{}"), Err(CheckpointError::Malformed)));
    assert!(matches!(CheckpointEnvelope::decode(&vec![b'x'; 32_769]), Err(CheckpointError::Oversized)));
}
