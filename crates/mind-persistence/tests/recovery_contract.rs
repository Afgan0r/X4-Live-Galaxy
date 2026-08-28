use mind_domain::{CommandId, MindAggregate, MindCommand, transition};
use mind_persistence::{
    CheckpointDraft, CheckpointEnvelope, CrashPoint, RecoveryDiagnostic, RecoveryInput,
    RecoveryOutcome, recover,
};
use observation_ingest::{AcceptedProjection, admit_batch};
use strategic_state::{Faction, PacketLimits, derive_packets};

const FRAMES: [&str; 4] = [
    r#"{"type":"observation","scope":"ZYA","entity_id":"ZYA:military:fleet","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#,
    r#"{"type":"observation","scope":"ARG","entity_id":"ARG:economy:ore","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#,
    r#"{"type":"observation","scope":"ZYA","entity_id":"ZYA:territorial:station","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#,
    r#"{"type":"observation","scope":"XEN","entity_id":"XEN:threat:XEN","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"shared"}"#,
];

fn checkpoint(report: &str) -> Result<CheckpointEnvelope, String> {
    let snapshot = admit_batch(AcceptedProjection::empty(), &FRAMES)
        .into_projection()
        .snapshot;
    let packets =
        derive_packets(&snapshot, PacketLimits::tracer()).map_err(|error| format!("{error:?}"))?;
    let packet = packets.packet(Faction::Zya);
    let commit = transition(
        &MindAggregate::empty(Faction::Zya),
        MindCommand::from_packet(packet, CommandId::new("mind-zya-1")),
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
fn recovers_acknowledged_identity_and_hides_each_crash_point() {
    let result = (checkpoint("report-zya-1"), checkpoint("report-zya-2"));
    assert!(result.0.is_ok() && result.1.is_ok());
    let (Ok(acknowledged), Ok(speculative)) = result else {
        return;
    };
    for point in [
        CrashPoint::BeforeX4Write,
        CrashPoint::AfterX4WriteBeforeAcknowledgement,
        CrashPoint::AfterAcknowledgementBeforeProjection,
    ] {
        let outcome = recover(RecoveryInput::crashed(
            acknowledged.clone(),
            speculative.clone(),
            point,
        ));
        assert_eq!(outcome.projection(), Some(&acknowledged));
        assert_ne!(outcome.projection(), Some(&speculative));
        assert_eq!(outcome.diagnostic(), None);
    }
}

#[test]
fn retains_last_valid_checkpoint_for_invalid_candidates_without_a_port_write() {
    let result = (checkpoint("report-zya-1"), checkpoint("report-zya-2"));
    assert!(result.0.is_ok() && result.1.is_ok());
    let (Ok(acknowledged), Ok(other)) = result else {
        return;
    };
    let encoded = other.encode();
    assert!(encoded.is_ok());
    let Ok(encoded) = encoded else { return };
    for candidate in [b"{}".to_vec(), b"partial".to_vec(), encoded] {
        let outcome = recover(RecoveryInput::candidate(acknowledged.clone(), candidate));
        assert_eq!(outcome.projection(), Some(&acknowledged));
        assert!(matches!(
            outcome.diagnostic(),
            Some(RecoveryDiagnostic::Rejected { .. })
        ));
        assert!(!outcome.port_write_requested());
    }
}

#[test]
fn unsupported_migration_returns_only_the_last_valid_projection() {
    let result = checkpoint("report-zya-1");
    assert!(result.is_ok());
    let Ok(acknowledged) = result else { return };
    let outcome = recover(RecoveryInput::migration(
        acknowledged.clone(),
        "unsupported-source",
        "v2",
    ));
    assert_eq!(
        outcome,
        RecoveryOutcome::retained(acknowledged, RecoveryDiagnostic::UnsupportedMigration)
    );
}
