#![expect(
    clippy::expect_used,
    reason = "test fixtures fail immediately when their invariants are invalid"
)]
use observation_domain::{
    BatchId, DecisionSnapshotId, ProducerIncarnationId, RecordId, SectionRevisionId, SourceScopeId,
    TransportEpoch,
};
use observation_ingest::{
    AmbiguityResolution, ApplicationContextIdentity, CompleteMessage, DeliveryStage,
    EnvelopeDecodeError, ImmutableApplicationBatch, ReceiverDisposition, SlotAdmission,
    SlotTurnover, StopAndWaitSlot, decode_complete_message,
};
use std::any::type_name;
const fn epoch(value: u64) -> TransportEpoch {
    TransportEpoch::new(value).expect("test epoch is positive")
}
fn batch(identity: &str, bytes: &[u8]) -> ImmutableApplicationBatch {
    let identity = BatchId::new(identity).expect("test batch identity is non-empty");
    ImmutableApplicationBatch::new(
        epoch(3),
        identity,
        bytes.to_vec(),
        ApplicationContextIdentity::Batch,
    )
    .expect("test batch is non-empty")
}
#[test]
fn immutable_batch_distinguishes_progress_handoff_and_volatile_receipt() {
    let mut slot = StopAndWaitSlot::empty();
    assert_eq!(slot.stage(), DeliveryStage::CollectionProgress);
    assert_eq!(
        slot.stage_batch(batch("batch:ship:7", b"ship-core")),
        SlotAdmission::Staged
    );
    assert_eq!(slot.mark_local_handoff(), Ok(DeliveryStage::LocalHandoff));
    assert_eq!(
        slot.apply_disposition(ReceiverDisposition::Received),
        Ok(DeliveryStage::Received)
    );
    assert_ne!(slot.stage(), DeliveryStage::Committed);
    assert_eq!(slot.pending_bytes(), Some(&b"ship-core"[..]));
}
#[test]
fn exact_replay_returns_prior_disposition_but_changed_bytes_conflict() {
    let mut slot = StopAndWaitSlot::empty();
    let original = batch("batch:ship:8", b"immutable");
    assert_eq!(slot.stage_batch(original.clone()), SlotAdmission::Staged);
    assert_eq!(slot.mark_local_handoff(), Ok(DeliveryStage::LocalHandoff));
    assert_eq!(
        slot.apply_disposition(ReceiverDisposition::Received),
        Ok(DeliveryStage::Received)
    );
    assert_eq!(
        slot.stage_batch(original),
        SlotAdmission::ExactReplay(ReceiverDisposition::Received)
    );
    assert_eq!(
        slot.stage_batch(batch("batch:ship:8", b"changed")),
        SlotAdmission::IdentityConflict
    );
    assert_eq!(slot.stage(), DeliveryStage::Received);
    assert_eq!(slot.pending_bytes(), Some(&b"immutable"[..]));
}
#[test]
fn non_consumption_and_terminal_outcomes_remain_distinct() {
    for (disposition, expected) in [
        (
            ReceiverDisposition::CapacityUnavailable,
            DeliveryStage::CollectionProgress,
        ),
        (
            ReceiverDisposition::TimedOutOrSuperseded,
            DeliveryStage::TerminalRejection,
        ),
        (
            ReceiverDisposition::StaleEpoch,
            DeliveryStage::TerminalRejection,
        ),
        (
            ReceiverDisposition::PermanentlyRejected,
            DeliveryStage::TerminalRejection,
        ),
        (
            ReceiverDisposition::AmbiguousCommit,
            DeliveryStage::AmbiguousPublication,
        ),
    ] {
        let mut slot = StopAndWaitSlot::empty();
        let pending = batch("batch:ship:9", b"pending");
        assert_eq!(slot.stage_batch(pending.clone()), SlotAdmission::Staged);
        assert_eq!(slot.mark_local_handoff(), Ok(DeliveryStage::LocalHandoff));
        assert_eq!(slot.apply_disposition(disposition), Ok(expected));
        assert_eq!(slot.pending_bytes(), Some(&b"pending"[..]));
        if disposition == ReceiverDisposition::CapacityUnavailable {
            assert_eq!(
                slot.stage_batch(pending),
                SlotAdmission::ExactReplay(ReceiverDisposition::CapacityUnavailable)
            );
        }
    }
}
#[test]
fn lifecycle_identifiers_have_distinct_compile_time_types() {
    let names = [
        type_name::<SourceScopeId>(),
        type_name::<ProducerIncarnationId>(),
        type_name::<TransportEpoch>(),
        type_name::<SectionRevisionId>(),
        type_name::<BatchId>(),
        type_name::<RecordId>(),
        type_name::<DecisionSnapshotId>(),
    ];
    for (index, left) in names.iter().enumerate() {
        assert!(names.iter().skip(index + 1).all(|right| left != right));
    }
}
#[test]
fn strict_whole_message_decode_rejects_invalid_contracts_before_staging() {
    let oversized = vec![b' '; 513];
    assert_eq!(
        decode_complete_message(&oversized, 512),
        Err(EnvelopeDecodeError::MessageTooLarge)
    );
    for invalid in [
        br#"{"type":"section_start"}"#.as_slice(),
        br#"{"type":"section_start","contract_version":2,"source_scope":"scope:ships","producer_incarnation":"producer:1","transport_epoch":1,"section_key":"ships","section_revision":1,"expected_records":1}"#.as_slice(),
        br#"{"type":"section_start","contract_version":1,"source_scope":"","producer_incarnation":"producer:1","transport_epoch":1,"section_key":"ships","section_revision":1,"expected_records":1}"#.as_slice(),
        br#"{"type":"section_start","contract_version":1,"source_scope":"scope:ships","producer_incarnation":"producer:1","transport_epoch":0,"section_key":"ships","section_revision":1,"expected_records":1}"#.as_slice(),
        br#"{"type":"section_start","contract_version":1,"source_scope":"scope:ships","producer_incarnation":"producer:1","transport_epoch":1,"section_key":"ships","section_revision":1,"expected_records":1,"unknown":true}"#.as_slice(),
    ] {
        assert!(decode_complete_message(invalid, 512).is_err());
    }
}
#[test]
fn ship_and_known_empty_station_use_the_same_strict_envelopes() {
    let ship = br#"{"type":"immutable_batch","contract_version":1,"source_scope":"scope:ships","producer_incarnation":"producer:1","transport_epoch":1,"section_key":"ships","section_revision":3,"batch_id":"batch:ships:3","records":[{"record_id":"record:ship:1","entity_id":"ship:1","observation_version":4,"content":"core"},{"record_id":"record:ship:2","entity_id":"ship:2","observation_version":2,"content":"core"}],"optional_detail":"detail_unavailable"}"#;
    let station = br#"{"type":"section_completion","contract_version":1,"source_scope":"scope:stations","producer_incarnation":"producer:1","transport_epoch":1,"section_key":"stations","section_revision":8,"record_count":0,"coverage":"known_empty"}"#;
    let CompleteMessage::ImmutableBatch(ship) =
        decode_complete_message(ship, 1024).expect("ship fixture is valid")
    else {
        panic!("expected immutable batch")
    };
    let CompleteMessage::SectionCompletion(station) =
        decode_complete_message(station, 1024).expect("station fixture is valid")
    else {
        panic!("expected section completion")
    };
    assert_eq!(ship.records.len(), 2);
    assert_eq!(ship.optional_detail.as_deref(), Some("detail_unavailable"));
    assert_eq!(station.record_count, 0);
    assert!(station.is_qualified_known_empty());
}
#[rustfmt::skip]
fn assert_definitive_turnover(disposition: ReceiverDisposition) {
    let mut slot = StopAndWaitSlot::empty();
    let original = batch("batch:turnover", b"exact-owned-bytes");
    let next = batch("batch:next", b"next");
    assert_eq!(slot.stage_batch(original.clone()), SlotAdmission::Staged);
    assert_eq!(slot.stage_batch(next.clone()), SlotAdmission::CapacityUnavailable);
    assert_eq!(slot.mark_local_handoff(), Ok(DeliveryStage::LocalHandoff));
    assert_eq!(slot.apply_disposition(disposition), Ok(slot.stage()));
    assert_eq!(slot.stage_batch(next.clone()), SlotAdmission::CapacityUnavailable);
    assert_eq!(slot.confirm_turnover(), Ok(SlotTurnover::Released));
    assert_eq!(slot.stage_batch(original), SlotAdmission::ExactReplay(disposition));
    assert_eq!(slot.stage_batch(batch("batch:turnover", b"changed")), SlotAdmission::IdentityConflict);
    assert_eq!(slot.stage_batch(next), SlotAdmission::Staged);
}
#[test] #[rustfmt::skip]
fn turnover_requires_confirmed_disposition_and_preserves_exact_replay() {
    [ReceiverDisposition::Received, ReceiverDisposition::Committed, ReceiverDisposition::TimedOutOrSuperseded, ReceiverDisposition::StaleEpoch, ReceiverDisposition::PermanentlyRejected]
        .into_iter().for_each(assert_definitive_turnover);
    let mut retained = StopAndWaitSlot::empty();
    let pending = batch("batch:retry", b"same-retry-bytes");
    assert_eq!(retained.stage_batch(pending.clone()), SlotAdmission::Staged);
    assert_eq!(retained.mark_local_handoff(), Ok(DeliveryStage::LocalHandoff));
    assert_eq!(retained.apply_disposition(ReceiverDisposition::CapacityUnavailable), Ok(DeliveryStage::CollectionProgress));
    assert_eq!(retained.confirm_turnover(), Ok(SlotTurnover::RetainedExactRetry));
    assert_eq!(retained.pending_bytes(), Some(pending.bytes()));
    assert_eq!(retained.mark_retry_handoff(), Ok(DeliveryStage::LocalHandoff));
}
#[test]
#[rustfmt::skip]
fn ambiguous_reconciliation_controls_retry_and_release() {
    let mut slot = StopAndWaitSlot::empty();
    let pending = batch("batch:ambiguous", b"one-uncertain-owner");
    assert_eq!(slot.stage_batch(pending.clone()), SlotAdmission::Staged);
    assert_eq!(slot.mark_local_handoff(), Ok(DeliveryStage::LocalHandoff));
    assert_eq!(slot.apply_disposition(ReceiverDisposition::AmbiguousCommit), Ok(DeliveryStage::AmbiguousPublication));
    assert_eq!(slot.stage_batch(batch("batch:blocked", b"blocked")), SlotAdmission::CapacityUnavailable);
    assert!(slot.mark_retry_handoff().is_err());
    assert_eq!(slot.confirm_turnover(), Ok(SlotTurnover::BlockedAmbiguous));
    assert_eq!(slot.apply_reconciliation(AmbiguityResolution::StillAmbiguous), Ok(SlotTurnover::BlockedAmbiguous));
    assert_eq!(slot.pending_bytes(), Some(pending.bytes()));
    assert_eq!(slot.apply_reconciliation(AmbiguityResolution::ProvenNotCommitted), Ok(SlotTurnover::RetainedExactRetry));
    assert_eq!(slot.pending_bytes(), Some(pending.bytes()));
    assert_eq!(slot.mark_retry_handoff(), Ok(DeliveryStage::LocalHandoff));
    assert_eq!(slot.apply_disposition(ReceiverDisposition::AmbiguousCommit), Ok(DeliveryStage::AmbiguousPublication));
    assert_eq!(slot.apply_reconciliation(AmbiguityResolution::Committed), Ok(SlotTurnover::Released));
    assert_eq!(slot.stage_batch(pending), SlotAdmission::ExactReplay(ReceiverDisposition::Committed));
}
