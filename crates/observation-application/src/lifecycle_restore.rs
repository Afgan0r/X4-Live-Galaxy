use observation_domain::SectionKey;
use observation_ingest::{DecisionEligibility, FinalizationOutcome};
use observation_persistence::{CurrentRevision, ObservationRepository};

use crate::ObservationLifecycle;

impl<R: ObservationRepository> ObservationLifecycle<R> {
    pub fn decision_eligibility(
        &self,
        required: &[SectionKey],
        now: u64,
        max_age: u64,
    ) -> DecisionEligibility {
        self.index.eligibility(required, now, max_age)
    }

    pub fn restore_current(&mut self, current: &CurrentRevision) -> bool {
        let revision = current.hydrate();
        for (key, value) in revision.context().dependencies() {
            self.index.record_current_pointer(key.clone(), *value);
        }
        if let Some(previous) = revision.context().expected_current() {
            self.index
                .record_current_pointer(revision.section_key().clone(), previous);
        }
        let Some(authority) = self.index.prepare_publication(revision) else {
            return false;
        };
        if !matches!(
            self.index
                .finalize_committed(&authority, current.receipt.accepted_at),
            FinalizationOutcome::Finalized | FinalizationOutcome::AlreadyFinalized
        ) {
            return false;
        }
        self.stager.record_committed_revision(authority.revision());
        true
    }
}
