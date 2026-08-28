use mind_domain::{CommandId, MindAggregate, MindCommand, transition};
use mind_persistence::{
    CheckpointDraft, CheckpointEnvelope, CheckpointPort, CompatibilityStatus, FakeCheckpointPort,
    PortError,
};
use observation_ingest::{AcceptedProjection, admit_batch};
use strategic_state::{Faction, PacketLimits, derive_packets};

const FRAMES: [&str; 4] = [
    r#"{"type":"observation","scope":"ZYA","entity_id":"ZYA:military:fleet","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#,
    r#"{"type":"observation","scope":"ARG","entity_id":"ARG:economy:ore","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#,
    r#"{"type":"observation","scope":"ZYA","entity_id":"ZYA:territorial:station","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#,
    r#"{"type":"observation","scope":"XEN","entity_id":"XEN:threat:XEN","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"shared"}"#,
];

fn envelope(report: &str) -> Result<CheckpointEnvelope, String> {
    let snapshot = admit_batch(AcceptedProjection::empty(), &FRAMES)
        .into_projection()
        .snapshot;
    let packets =
        derive_packets(&snapshot, PacketLimits::tracer()).map_err(|error| format!("{error:?}"))?;
    let commit = transition(
        &MindAggregate::empty(Faction::Zya),
        MindCommand::from_packet(packets.packet(Faction::Zya), CommandId::new("mind-zya-1")),
    )
    .map_err(|error| format!("{error:?}"))?;
    CheckpointEnvelope::from_pending_commit(
        1,
        None,
        &commit,
        CheckpointDraft::new(
            "snapshot-zya-1",
            "tick-zya-1",
            "replay-zya-1",
            "admission-zya-1",
            report,
        ),
    )
    .map_err(|error| format!("{error:?}"))
}

#[test]
fn acknowledges_exact_retry_without_duplicate_tick_or_report() {
    let candidate_result = envelope("report-zya-1");
    assert!(candidate_result.is_ok());
    let Ok(candidate) = candidate_result else {
        return;
    };
    let mut port = FakeCheckpointPort::new();
    let first = port.compare_and_set(None, candidate.clone());
    assert!(first.is_ok());
    let retry = port.compare_and_set(None, candidate);
    assert_eq!(first, retry);
    let Ok(ack) = first else { return };
    let reloaded = port.load();
    assert!(reloaded.is_some());
    let Some(reloaded) = reloaded else { return };
    assert_eq!(reloaded.cursor(), ack.cursor);
    assert_eq!(reloaded.strategic_tick_id(), ack.strategic_tick_identity);
    assert_eq!(reloaded.reserved_report_id(), ack.reserved_report_identity);
    assert_eq!(port.reread_ack(&ack.cursor), Ok(ack));
}

#[test]
fn rejects_content_collision_and_requires_x4_restart_on_protocol_mismatch() {
    let first_result = envelope("report-zya-1");
    let collision_result = envelope("report-zya-2");
    assert!(first_result.is_ok() && collision_result.is_ok());
    let (Ok(first), Ok(collision)) = (first_result, collision_result) else {
        return;
    };
    let mut port = FakeCheckpointPort::new();
    let ack = port.compare_and_set(None, first);
    assert!(ack.is_ok());
    assert_eq!(
        port.compare_and_set(None, collision),
        Err(PortError::ContentCollision)
    );
    assert_eq!(
        port.compatibility("other.protocol"),
        CompatibilityStatus::X4RestartRequired
    );
    assert!(port.load().is_some());
}
