use std::collections::BTreeMap;

use observation_domain::{SectionKey, SourceScopeId, SourceSessionIdentity};
use observation_ingest::DecisionEligibility;
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

    pub fn restore_current_snapshot(&mut self, currents: &[CurrentRevision]) -> bool {
        if !sessions_are_consistent(currents) {
            return false;
        }
        if !self
            .index
            .install_current_snapshot(currents.iter().map(|current| {
                (
                    current.revision.section_key.clone(),
                    current.revision.revision,
                )
            }))
        {
            return false;
        }
        currents.iter().all(|current| self.restore_one(current))
    }

    fn restore_one(&mut self, current: &CurrentRevision) -> bool {
        let revision = current.hydrate();
        if !self
            .index
            .restore_committed(revision.clone(), current.receipt.accepted_at)
        {
            return false;
        }
        self.stager.record_committed_revision(&revision);
        true
    }
}

fn sessions_are_consistent(currents: &[CurrentRevision]) -> bool {
    let mut sessions: BTreeMap<&SourceScopeId, &SourceSessionIdentity> = BTreeMap::new();
    currents.iter().all(|current| {
        let revision = &current.revision;
        sessions
            .insert(&revision.source_scope, &revision.source_session)
            .is_none_or(|existing| existing == &revision.source_session)
    })
}
