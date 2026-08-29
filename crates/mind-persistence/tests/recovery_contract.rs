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
    checkpoint_at(1, None, report)
}

fn checkpoint_at(
    sequence: u64,
    predecessor: Option<&mind_persistence::CheckpointCursor>,
    report: &str,
) -> Result<CheckpointEnvelope, String> {
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
        sequence,
        predecessor.cloned(),
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
        Some(acknowledged.clone()),
        "unsupported-source",
        "v2",
        Vec::new(),
    ));
    assert_eq!(
        outcome,
        RecoveryOutcome::retained(acknowledged, RecoveryDiagnostic::UnsupportedMigration)
    );
}

#[test]
fn ordered_legacy_migration_exposes_only_a_valid_target_copy() {
    let outcome = recover(RecoveryInput::migration(
        None,
        "mind-checkpoint-v0",
        "1",
        br#"{"sequence":1,"snapshot":"snapshot-zya-1","tick":"tick-zya-1","mind":"legacy","replay":"replay-zya-1","admission":"admission-zya-1","report":"report-zya-1"}"#.to_vec(),
    ));
    assert!(outcome.projection().is_some());
    assert_eq!(outcome.diagnostic(), None);
    assert!(outcome.port_write_requested());
}

#[test]
fn invalid_legacy_payloads_retain_fallback_without_a_write() {
    let fallback = checkpoint("report-zya-1");
    assert!(fallback.is_ok());
    let Ok(fallback) = fallback else { return };
    let partial = br#"{"sequence":1,"snapshot":"snapshot-zya-1"}"#.to_vec();
    let unknown = br#"{"sequence":1,"snapshot":"s","tick":"t","mind":"m","replay":"r","admission":"a","report":"p","extra":true}"#.to_vec();
    let oversized = vec![b'x'; 32_769];

    for legacy in [b"not-json".to_vec(), partial, unknown, oversized] {
        let outcome = recover(RecoveryInput::migration(
            Some(fallback.clone()),
            "mind-checkpoint-v0",
            "1",
            legacy,
        ));
        assert_eq!(outcome.projection(), Some(&fallback));
        assert!(matches!(
            outcome.diagnostic(),
            Some(RecoveryDiagnostic::Rejected {
                code: "invalid-legacy"
            })
        ));
        assert!(!outcome.port_write_requested());
    }
}

#[test]
fn rejects_duplicate_stale_and_out_of_order_candidates() {
    let first = checkpoint("report-zya-1");
    assert!(first.is_ok());
    let Ok(first) = first else { return };
    let second = checkpoint_at(2, Some(&first.cursor()), "report-zya-2");
    assert!(second.is_ok());
    let Ok(second) = second else { return };
    let duplicate = checkpoint("report-zya-collision");
    let stale = first.encode();
    let third = checkpoint_at(3, Some(&second.cursor()), "report-zya-3");
    assert!(duplicate.is_ok() && stale.is_ok() && third.is_ok());
    let (Ok(duplicate), Ok(stale), Ok(third)) = (duplicate, stale, third) else {
        return;
    };
    let duplicate = duplicate.encode();
    let out_of_order = third.encode();
    assert!(duplicate.is_ok() && out_of_order.is_ok());
    let (Ok(duplicate), Ok(out_of_order)) = (duplicate, out_of_order) else {
        return;
    };
    for candidate in [duplicate, stale, out_of_order] {
        let outcome = recover(RecoveryInput::candidate(second.clone(), candidate));
        assert_eq!(outcome.projection(), Some(&second));
        assert!(matches!(
            outcome.diagnostic(),
            Some(RecoveryDiagnostic::Rejected { .. })
        ));
    }
}
