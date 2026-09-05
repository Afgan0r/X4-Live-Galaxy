use observation_domain::SectionCompletionEnvelope;
use observation_ingest::{
    AmbiguityResolution, CompletionCurrent, CompletionOutcome, ReceiverDisposition, RejectionReason,
};
use observation_persistence::{
    ObservationRepository, PublishOutcome, PublishRequest, ReconciliationOutcome,
};

use crate::{
    AttemptState, LifecycleError, LifecycleResult, ObservationLifecycle, PublicationReconciler,
    ReconcileResult, RetainedPublicationAttempt,
};

impl<R: ObservationRepository> ObservationLifecycle<R> {
    pub fn reconcile_ambiguous(&mut self, now: u64) -> Result<LifecycleResult, LifecycleError>
    where
        R: PublicationReconciler,
    {
        let Some(mut attempt) = self.retained.take() else {
            return Err(LifecycleError::RetryNotEligible);
        };
        if !attempt.reconciliation_allowed(self.limits, now) {
            self.retained = Some(attempt);
            return Ok(RetainedPublicationAttempt::reconciled(
                ReconcileResult::StillAmbiguous,
            ));
        }
        attempt.record_reconciliation();
        match self.repository.reconcile_publication(&attempt.request) {
            ReconciliationOutcome::CommittedReplay(_) => self.finish_reconciliation(attempt),
            ReconciliationOutcome::ProvenNotCommitted => {
                let _ = self
                    .slot
                    .apply_reconciliation(AmbiguityResolution::ProvenNotCommitted)
                    .map_err(|_| LifecycleError::SlotInvariant)?;
                attempt.state = AttemptState::RetryEligible;
                self.retained = Some(attempt);
                Ok(RetainedPublicationAttempt::reconciled(
                    ReconcileResult::ProvenNotCommitted,
                ))
            }
            ReconciliationOutcome::Superseded(_) => {
                if self
                    .slot
                    .apply_reconciliation(AmbiguityResolution::Superseded)
                    .is_err()
                {
                    self.retained = Some(attempt);
                    return Err(LifecycleError::SlotInvariant);
                }
                Ok(LifecycleResult::Disposition(
                    ReceiverDisposition::TimedOutOrSuperseded,
                ))
            }
            ReconciliationOutcome::Ambiguous(_) => {
                self.retained = Some(attempt);
                Ok(RetainedPublicationAttempt::reconciled(
                    ReconcileResult::StillAmbiguous,
                ))
            }
        }
    }

    pub fn retry_proven_not_committed(&mut self) -> Result<LifecycleResult, LifecycleError> {
        let Some(attempt) = self.retained.take() else {
            return Err(LifecycleError::RetryNotEligible);
        };
        if attempt.state != AttemptState::RetryEligible {
            self.retained = Some(attempt);
            return Err(LifecycleError::RetryNotEligible);
        }
        let _ = self
            .slot
            .mark_retry_handoff()
            .map_err(|_| LifecycleError::SlotInvariant)?;
        self.publish_attempt(attempt)
    }

    pub(crate) fn complete(
        &mut self,
        completion: SectionCompletionEnvelope,
        current: &CompletionCurrent,
        now: u64,
    ) -> Result<LifecycleResult, LifecycleError> {
        let Some(certificate) = self.stager.completion_certificate(completion) else {
            return self.finish_disposition(ReceiverDisposition::PermanentlyRejected);
        };
        let revision = match self.stager.complete_section(&certificate, current, now) {
            CompletionOutcome::Validated(revision) => *revision,
            CompletionOutcome::Rejected(reason) => {
                return self.finish_disposition(rejection_disposition(reason));
            }
        };
        let Some(authority) = self.index.prepare_publication(revision) else {
            return self.finish_disposition(ReceiverDisposition::PermanentlyRejected);
        };
        let request = PublishRequest::from_accepted(authority.clone(), now);
        let Some(retained_bytes) = request
            .retained_bytes()
            .and_then(|bytes| self.slot.pending_bytes()?.len().checked_add(bytes))
        else {
            return self.finish_disposition(ReceiverDisposition::PermanentlyRejected);
        };
        if retained_bytes > self.limits.retained_attempt_bytes.get() {
            return self.finish_disposition(ReceiverDisposition::PermanentlyRejected);
        }
        self.publish_attempt(RetainedPublicationAttempt::new(
            request,
            authority,
            now,
            retained_bytes,
        ))
    }

    fn publish_attempt(
        &mut self,
        attempt: RetainedPublicationAttempt,
    ) -> Result<LifecycleResult, LifecycleError> {
        match self.repository.publish(attempt.request.clone()) {
            PublishOutcome::CommittedNew(_) | PublishOutcome::CommittedReplay(_) => {
                self.finish_committed(attempt)
            }
            PublishOutcome::Ambiguous(_) => {
                self.block_attempt(attempt, LifecycleError::BlockedAmbiguous)
            }
            PublishOutcome::StalePointer(_) | PublishOutcome::StaleDependency(_) => {
                self.finish_disposition(ReceiverDisposition::TimedOutOrSuperseded)
            }
            PublishOutcome::Conflict(_) | PublishOutcome::PermanentRejection(_) => {
                self.finish_disposition(ReceiverDisposition::PermanentlyRejected)
            }
        }
    }

    fn finish_committed(
        &mut self,
        attempt: RetainedPublicationAttempt,
    ) -> Result<LifecycleResult, LifecycleError> {
        let outcome = self
            .index
            .finalize_committed(&attempt.authority, attempt.accepted_at);
        if RetainedPublicationAttempt::finalized(outcome) {
            self.stager
                .record_committed_revision(attempt.authority.revision());
            return self.finish_disposition(ReceiverDisposition::Committed);
        }
        self.block_attempt(attempt, LifecycleError::FinalizationBlocked)
    }

    fn block_attempt(
        &mut self,
        attempt: RetainedPublicationAttempt,
        error: LifecycleError,
    ) -> Result<LifecycleResult, LifecycleError> {
        let _ = self
            .slot
            .apply_disposition(ReceiverDisposition::AmbiguousCommit)
            .and_then(|_| self.slot.confirm_turnover())
            .map_err(|_| LifecycleError::SlotInvariant)?;
        self.retained = Some(attempt);
        if error == LifecycleError::BlockedAmbiguous {
            Ok(LifecycleResult::Disposition(
                ReceiverDisposition::AmbiguousCommit,
            ))
        } else {
            Err(error)
        }
    }

    fn finish_reconciliation(
        &mut self,
        attempt: RetainedPublicationAttempt,
    ) -> Result<LifecycleResult, LifecycleError> {
        let outcome = self
            .index
            .finalize_committed(&attempt.authority, attempt.accepted_at);
        if !RetainedPublicationAttempt::finalized(outcome) {
            self.retained = Some(attempt);
            return Err(LifecycleError::FinalizationBlocked);
        }
        self.stager
            .record_committed_revision(attempt.authority.revision());
        let _ = self
            .slot
            .apply_reconciliation(AmbiguityResolution::Committed)
            .map_err(|_| LifecycleError::SlotInvariant)?;
        Ok(RetainedPublicationAttempt::reconciled(
            ReconcileResult::Committed,
        ))
    }
}

const fn rejection_disposition(reason: RejectionReason) -> ReceiverDisposition {
    if matches!(reason, RejectionReason::DependencyChanged) {
        ReceiverDisposition::TimedOutOrSuperseded
    } else {
        ReceiverDisposition::PermanentlyRejected
    }
}
