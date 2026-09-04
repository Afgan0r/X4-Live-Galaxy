use super::{
    DeliveryStage, ReceiverDisposition,
    slot_transition::{classify, stage_for},
};
use observation_domain::{BatchId, TransportEpoch};
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
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SlotTurnover {
    Released,
    RetainedExactRetry,
    BlockedAmbiguous,
}
#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AmbiguityResolution {
    Committed,
    ProvenNotCommitted,
    StillAmbiguous,
}
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FeedbackError {
    NoPendingBatch,
    HandoffRequired,
    DispositionRequired,
    AmbiguityRequired,
    RetryNotEligible,
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct PendingBatch {
    batch: ImmutableApplicationBatch,
    disposition: Option<ReceiverDisposition>,
}
#[derive(Clone, Debug, Eq, PartialEq)]
struct ReleasedReplay {
    batch: ImmutableApplicationBatch,
    disposition: ReceiverDisposition,
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StopAndWaitSlot {
    stage: DeliveryStage,
    pending: Option<PendingBatch>,
    released: Option<ReleasedReplay>,
}

impl StopAndWaitSlot {
    pub const fn empty() -> Self {
        Self {
            stage: DeliveryStage::CollectionProgress,
            pending: None,
            released: None,
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
        if let Some(pending) = &self.pending {
            return classify(&pending.batch, pending.disposition, &batch, true);
        }
        match self
            .released
            .as_ref()
            .map(|held| classify(&held.batch, Some(held.disposition), &batch, false))
        {
            None | Some(SlotAdmission::Staged) => {}
            Some(replay) => return replay,
        }
        self.pending = Some(PendingBatch {
            batch,
            disposition: None,
        });
        self.released = None;
        self.stage = DeliveryStage::CollectionProgress;
        SlotAdmission::Staged
    }

    pub fn mark_local_handoff(&mut self) -> Result<DeliveryStage, FeedbackError> {
        let pending = self.pending.as_ref().ok_or(FeedbackError::NoPendingBatch)?;
        if pending.disposition.is_some() || self.stage != DeliveryStage::CollectionProgress {
            return Err(FeedbackError::HandoffRequired);
        }
        self.stage = DeliveryStage::LocalHandoff;
        Ok(self.stage)
    }

    pub fn mark_retry_handoff(&mut self) -> Result<DeliveryStage, FeedbackError> {
        let Some(pending) = &self.pending else {
            return Err(FeedbackError::NoPendingBatch);
        };
        if pending.disposition != Some(ReceiverDisposition::CapacityUnavailable)
            || self.stage != DeliveryStage::CollectionProgress
        {
            return Err(FeedbackError::RetryNotEligible);
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
        self.stage = stage_for(disposition);
        Ok(self.stage)
    }

    pub fn confirm_turnover(&mut self) -> Result<SlotTurnover, FeedbackError> {
        let disposition = self
            .pending
            .as_ref()
            .ok_or(FeedbackError::NoPendingBatch)?
            .disposition
            .ok_or(FeedbackError::DispositionRequired)?;
        match disposition {
            ReceiverDisposition::CapacityUnavailable => Ok(SlotTurnover::RetainedExactRetry),
            ReceiverDisposition::AmbiguousCommit => Ok(SlotTurnover::BlockedAmbiguous),
            _ => self.release(disposition),
        }
    }

    pub fn apply_reconciliation(
        &mut self,
        resolution: AmbiguityResolution,
    ) -> Result<SlotTurnover, FeedbackError> {
        if self.stage != DeliveryStage::AmbiguousPublication {
            return Err(FeedbackError::AmbiguityRequired);
        }
        match resolution {
            AmbiguityResolution::Committed => self.release(ReceiverDisposition::Committed),
            AmbiguityResolution::ProvenNotCommitted => {
                let pending = self.pending.as_mut().ok_or(FeedbackError::NoPendingBatch)?;
                pending.disposition = Some(ReceiverDisposition::CapacityUnavailable);
                self.stage = DeliveryStage::CollectionProgress;
                Ok(SlotTurnover::RetainedExactRetry)
            }
            AmbiguityResolution::StillAmbiguous => Ok(SlotTurnover::BlockedAmbiguous),
        }
    }

    fn release(&mut self, disposition: ReceiverDisposition) -> Result<SlotTurnover, FeedbackError> {
        let pending = self.pending.take().ok_or(FeedbackError::NoPendingBatch)?;
        self.released = Some(ReleasedReplay {
            batch: pending.batch,
            disposition,
        });
        self.stage = stage_for(disposition);
        Ok(SlotTurnover::Released)
    }
}
