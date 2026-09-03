#![expect(
    clippy::expect_used,
    reason = "test fixtures fail immediately when their invariants are invalid"
)]

use std::any::type_name;

use observation_domain::{
    BatchId, DecisionSnapshotId, ProducerIncarnationId, RecordId, SectionRevisionId, SourceScopeId,
    TransportEpoch,
};
use observation_ingest::{
    DeliveryStage, ImmutableApplicationBatch, ReceiverDisposition, SlotAdmission, StopAndWaitSlot,
};

const fn epoch(value: u64) -> TransportEpoch {
    TransportEpoch::new(value).expect("test epoch is positive")
}

fn batch_id(value: &str) -> BatchId {
    BatchId::new(value).expect("test batch identity is non-empty")
}

fn batch(identity: &str, bytes: &[u8]) -> ImmutableApplicationBatch {
    ImmutableApplicationBatch::new(epoch(3), batch_id(identity), bytes.to_vec())
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
