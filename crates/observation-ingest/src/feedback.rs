use observation_domain::{BatchId, TransportEpoch};

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DeliveryStage {
    CollectionProgress,
    LocalHandoff,
    Received,
    Committed,
    TerminalRejection,
    AmbiguousPublication,
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ReceiverDisposition {
    CapacityUnavailable,
    Received,
    Committed,
    TimedOutOrSuperseded,
    StaleEpoch,
    PermanentlyRejected,
    AmbiguousCommit,
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ImmutableApplicationBatch {
    epoch: TransportEpoch,
    identity: BatchId,
    bytes: Vec<u8>,
}

impl ImmutableApplicationBatch {
    #[must_use]
    pub fn new(epoch: TransportEpoch, identity: BatchId, bytes: Vec<u8>) -> Option<Self> {
        (!bytes.is_empty()).then_some(Self {
            epoch,
            identity,
            bytes,
        })
    }

    pub const fn epoch(&self) -> TransportEpoch {
        self.epoch
    }

    pub const fn identity(&self) -> &BatchId {
        &self.identity
    }

    #[must_use]
    pub fn bytes(&self) -> &[u8] {
        &self.bytes
    }
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SlotAdmission {
    Staged,
    AlreadyStaged,
    ExactReplay(ReceiverDisposition),
    IdentityConflict,
    CapacityUnavailable,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FeedbackError {
    NoPendingBatch,
    HandoffRequired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
struct PendingBatch {
    batch: ImmutableApplicationBatch,
    disposition: Option<ReceiverDisposition>,
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StopAndWaitSlot {
    stage: DeliveryStage,
    pending: Option<PendingBatch>,
}

impl StopAndWaitSlot {
    pub const fn empty() -> Self {
        Self {
            stage: DeliveryStage::CollectionProgress,
            pending: None,
        }
    }

    pub const fn stage(&self) -> DeliveryStage {
        self.stage
    }

    #[must_use]
    pub fn pending_bytes(&self) -> Option<&[u8]> {
        self.pending.as_ref().map(|pending| pending.batch.bytes())
    }

    pub fn stage_batch(&mut self, batch: ImmutableApplicationBatch) -> SlotAdmission {
        let Some(pending) = &self.pending else {
            self.pending = Some(PendingBatch {
                batch,
                disposition: None,
            });
            self.stage = DeliveryStage::CollectionProgress;
            return SlotAdmission::Staged;
        };
        if pending.batch.identity() != batch.identity() || pending.batch.epoch() != batch.epoch() {
            return SlotAdmission::CapacityUnavailable;
        }
        if pending.batch.bytes() != batch.bytes() {
            return SlotAdmission::IdentityConflict;
        }
        match pending.disposition {
            Some(disposition) => SlotAdmission::ExactReplay(disposition),
            None => SlotAdmission::AlreadyStaged,
        }
    }

    pub const fn mark_local_handoff(&mut self) -> Result<DeliveryStage, FeedbackError> {
        if self.pending.is_none() {
            return Err(FeedbackError::NoPendingBatch);
        }
        self.stage = DeliveryStage::LocalHandoff;
        Ok(self.stage)
    }

    pub fn apply_disposition(
        &mut self,
        disposition: ReceiverDisposition,
    ) -> Result<DeliveryStage, FeedbackError> {
        let pending = self.pending.as_mut().ok_or(FeedbackError::NoPendingBatch)?;
        if self.stage != DeliveryStage::LocalHandoff {
            return Err(FeedbackError::HandoffRequired);
        }
        pending.disposition = Some(disposition);
        self.stage = match disposition {
            ReceiverDisposition::CapacityUnavailable => DeliveryStage::CollectionProgress,
            ReceiverDisposition::Received => DeliveryStage::Received,
            ReceiverDisposition::Committed => DeliveryStage::Committed,
            ReceiverDisposition::TimedOutOrSuperseded
            | ReceiverDisposition::StaleEpoch
            | ReceiverDisposition::PermanentlyRejected => DeliveryStage::TerminalRejection,
            ReceiverDisposition::AmbiguousCommit => DeliveryStage::AmbiguousPublication,
        };
        Ok(self.stage)
    }
}
