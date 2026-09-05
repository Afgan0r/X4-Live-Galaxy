use observation_ingest::{AmbiguityResolution, ReceiverDisposition};
use observation_persistence::ObservationRepository;

use crate::{LifecycleError, LifecycleResult, ObservationLifecycle, RetainedPublicationAttempt};

impl<R: ObservationRepository> ObservationLifecycle<R> {
    pub(super) fn finish_superseded(
        &mut self,
        attempt: RetainedPublicationAttempt,
    ) -> Result<LifecycleResult, LifecycleError> {
        if self
            .slot
            .apply_reconciliation(AmbiguityResolution::Superseded)
            .is_ok()
        {
            Ok(LifecycleResult::Disposition(
                ReceiverDisposition::TimedOutOrSuperseded,
            ))
        } else {
            self.retained = Some(attempt);
            Err(LifecycleError::SlotInvariant)
        }
    }
}
