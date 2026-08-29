use mind_domain::{
    Capability, CommandId, InitiativeCommand, InitiativeId, InitiativeSpec, MindAggregate,
    MindCommand, transition,
};
use mind_persistence::{CheckpointDraft, CheckpointEnvelope, CheckpointError};
use observation_ingest::{AcceptedProjection, admit_batch};
use strategic_state::{Faction, PacketLimits, derive_packets};

const FRAMES: [&str; 4] = [
    r#"{"type":"observation","scope":"ZYA","entity_id":"ZYA:military:fleet","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#,
    r#"{"type":"observation","scope":"ARG","entity_id":"ARG:economy:ore","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#,
    r#"{"type":"observation","scope":"ZYA","entity_id":"ZYA:territorial:station","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#,
    r#"{"type":"observation","scope":"XEN","entity_id":"XEN:threat:XEN","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"shared"}"#,
];

fn committed_mind() -> Result<mind_domain::PendingMindCommit, String> {
    let snapshot = admit_batch(AcceptedProjection::empty(), &FRAMES)
        .into_projection()
        .snapshot;
    let packets =
        derive_packets(&snapshot, PacketLimits::tracer()).map_err(|error| format!("{error:?}"))?;
    let packet = packets.packet(Faction::Zya);
    let initial = transition(
        &MindAggregate::empty(Faction::Zya),
        MindCommand::from_packet(packet, CommandId::new("mind-zya-1")),
    )
    .map_err(|error| format!("{error:?}"))?;
    let initiative = initial
        .aggregate()
        .apply_initiative(InitiativeCommand::accept(
            CommandId::new("initiative-command-zya-1"),
            InitiativeSpec::new(
                InitiativeId::new("initiative-zya-1"),
                Capability::DefenseAndMilitaryStrategy,
                "defend frontier",
                "military-fact",
                90,
            ),
        ))
        .map_err(|error| format!("{error:?}"))?;
    transition(
        initiative.aggregate(),
        MindCommand::from_packet(packet, CommandId::new("mind-zya-2")),
    )
    .map_err(|error| format!("{error:?}"))
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
    let commit_result = committed_mind();
    assert!(commit_result.is_ok());
    let Ok(commit) = commit_result else { return };
    let first_result = CheckpointEnvelope::from_pending_commit(1, None, &commit, draft());
    assert!(first_result.is_ok());
    let Ok(first) = first_result else { return };
    let second_result = CheckpointEnvelope::from_pending_commit(1, None, &commit, draft());
    assert!(second_result.is_ok());
    let Ok(second) = second_result else { return };

    let first_bytes = first.encode();
    let second_bytes = second.encode();
    assert_eq!(first_bytes, second_bytes);
    assert_eq!(first.integrity_hash(), second.integrity_hash());
    assert_eq!(first.sequence(), 1);
    assert_eq!(first.strategic_tick_id(), "tick-zya-1");
    assert_eq!(first.restored_mind(), Ok(commit.aggregate().clone()));
    let Ok(bytes) = first_bytes else { return };
    let decoded = CheckpointEnvelope::decode(&bytes);
    assert_eq!(decoded, Ok(first.clone()));
    assert_eq!(
        decoded.and_then(|value| value.restored_mind()),
        first.restored_mind()
    );
}

#[test]
fn rejects_tampered_partial_and_oversized_checkpoint_records() {
    let commit_result = committed_mind();
    assert!(commit_result.is_ok());
    let Ok(commit) = commit_result else { return };
    let envelope_result = CheckpointEnvelope::from_pending_commit(1, None, &commit, draft());
    assert!(envelope_result.is_ok());
    let Ok(envelope) = envelope_result else {
        return;
    };
    let bytes_result = envelope.encode();
    assert!(bytes_result.is_ok());
    let Ok(bytes) = bytes_result else { return };
    let text_result = String::from_utf8(bytes);
    assert!(text_result.is_ok());
    let Ok(text) = text_result else { return };
    let encoded = text
        .replace("snapshot-zya-1", "snapshot-zya-2")
        .into_bytes();
    assert_eq!(
        CheckpointEnvelope::decode(&encoded),
        Err(CheckpointError::InvalidHash)
    );
    assert!(matches!(
        CheckpointEnvelope::decode(b"{}"),
        Err(CheckpointError::Malformed)
    ));
    assert!(matches!(
        CheckpointEnvelope::decode(&vec![b'x'; 32_769]),
        Err(CheckpointError::Oversized)
    ));
}
