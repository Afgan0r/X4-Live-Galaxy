use super::{DeliveryStage, ImmutableApplicationBatch, ReceiverDisposition, slot::SlotAdmission};

pub(super) fn classify(
    held: &ImmutableApplicationBatch,
    disposition: Option<ReceiverDisposition>,
    incoming: &ImmutableApplicationBatch,
    occupied: bool,
) -> SlotAdmission {
    if held.identity() != incoming.identity() || held.epoch() != incoming.epoch() {
        return if occupied {
            SlotAdmission::CapacityUnavailable
        } else {
            SlotAdmission::Staged
        };
    }
    if held.bytes() != incoming.bytes() || held.context() != incoming.context() {
        return SlotAdmission::IdentityConflict;
    }
    disposition.map_or(SlotAdmission::AlreadyStaged, SlotAdmission::ExactReplay)
}

pub(super) const fn stage_for(disposition: ReceiverDisposition) -> DeliveryStage {
    match disposition {
        ReceiverDisposition::CapacityUnavailable => DeliveryStage::CollectionProgress,
        ReceiverDisposition::Received => DeliveryStage::Received,
        ReceiverDisposition::Committed => DeliveryStage::Committed,
        ReceiverDisposition::TimedOutOrSuperseded
        | ReceiverDisposition::StaleEpoch
        | ReceiverDisposition::PermanentlyRejected => DeliveryStage::TerminalRejection,
        ReceiverDisposition::AmbiguousCommit => DeliveryStage::AmbiguousPublication,
    }
}
