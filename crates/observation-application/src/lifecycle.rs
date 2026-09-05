use observation_domain::CompleteMessage;
use observation_ingest::{
    DecisionRevisionIndex, GenerationStager, ImmutableApplicationBatch, ReceiverDisposition,
    SlotAdmission, StopAndWaitSlot, decode_complete_message,
};
use observation_persistence::ObservationRepository;

use crate::{
    LifecycleContext, LifecycleError, LifecycleInput, LifecycleLimits, LifecycleResult,
    RetainedPublicationAttempt,
};

pub struct ObservationLifecycle<R> {
    pub(crate) slot: StopAndWaitSlot,
    pub(crate) stager: GenerationStager,
    pub(crate) index: DecisionRevisionIndex,
    pub(crate) repository: R,
    pub(crate) retained: Option<RetainedPublicationAttempt>,
    pub(crate) limits: LifecycleLimits,
}

impl<R: ObservationRepository> ObservationLifecycle<R> {
    pub const fn new(
        stager: GenerationStager,
        index: DecisionRevisionIndex,
        repository: R,
        limits: LifecycleLimits,
    ) -> Self {
        Self {
            slot: StopAndWaitSlot::empty(),
            stager,
            index,
            repository,
            retained: None,
            limits,
        }
    }

    pub fn submit(&mut self, input: LifecycleInput) -> Result<LifecycleResult, LifecycleError> {
        if self.retained.is_some() {
            return Err(LifecycleError::BlockedAmbiguous);
        }
        let message =
            decode_complete_message(&input.bytes, self.limits.complete_message_bytes.get())
                .map_err(|_| LifecycleError::DecodeRejected)?;
        validate_context(&message, &input.context, input.epoch)?;
        let replay_identity = input.context.replay_identity(input.work);
        let batch = ImmutableApplicationBatch::new(
            input.epoch,
            input.identity,
            input.bytes,
            replay_identity,
        )
        .ok_or(LifecycleError::DecodeRejected)?;
        match self.slot.stage_batch(batch) {
            SlotAdmission::Staged => {}
            SlotAdmission::ExactReplay(disposition) => {
                return Ok(LifecycleResult::Disposition(disposition));
            }
            SlotAdmission::AlreadyStaged => return Err(LifecycleError::SlotInvariant),
            SlotAdmission::IdentityConflict => {
                return Ok(LifecycleResult::Disposition(
                    ReceiverDisposition::PermanentlyRejected,
                ));
            }
            SlotAdmission::CapacityUnavailable => {
                return Ok(LifecycleResult::Disposition(
                    ReceiverDisposition::CapacityUnavailable,
                ));
            }
        }
        let _ = self
            .slot
            .mark_local_handoff()
            .map_err(|_| LifecycleError::SlotInvariant)?;
        self.dispatch(message, input.context, input.work, input.now)
    }

    fn dispatch(
        &mut self,
        message: CompleteMessage,
        context: LifecycleContext,
        work: usize,
        now: u64,
    ) -> Result<LifecycleResult, LifecycleError> {
        let disposition = match (message, context) {
            (CompleteMessage::SectionStart(start), LifecycleContext::Start(context)) => {
                self.stager.start_section_with_context(start, context, now)
            }
            (CompleteMessage::ImmutableBatch(batch), LifecycleContext::Batch) => {
                self.stager.stage_section_batch(batch, work, now)
            }
            (
                CompleteMessage::SectionCompletion(completion),
                LifecycleContext::Completion(current),
            ) => return self.complete(completion, &current, now),
            _ => return Err(LifecycleError::ContextMismatch),
        };
        self.finish_disposition(disposition)
    }

    pub(crate) fn finish_disposition(
        &mut self,
        disposition: ReceiverDisposition,
    ) -> Result<LifecycleResult, LifecycleError> {
        let _ = self
            .slot
            .apply_disposition(disposition)
            .and_then(|_| self.slot.confirm_turnover())
            .map_err(|_| LifecycleError::SlotInvariant)?;
        Ok(LifecycleResult::Disposition(disposition))
    }
}

fn validate_context(
    message: &CompleteMessage,
    context: &LifecycleContext,
    epoch: observation_domain::TransportEpoch,
) -> Result<(), LifecycleError> {
    let valid = match (message, context) {
        (CompleteMessage::SectionStart(value), LifecycleContext::Start(_)) => {
            value.transport_epoch == epoch
        }
        (CompleteMessage::ImmutableBatch(value), LifecycleContext::Batch) => {
            value.transport_epoch == epoch
        }
        (CompleteMessage::SectionCompletion(value), LifecycleContext::Completion(_)) => {
            value.transport_epoch == epoch
        }
        _ => false,
    };
    valid.then_some(()).ok_or(LifecycleError::ContextMismatch)
}
