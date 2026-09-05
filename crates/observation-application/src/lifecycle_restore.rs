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

    pub fn restore_current_snapshot(&mut self) -> bool {
        let Ok(currents) = self.repository.current_snapshot() else {
            return false;
        };
        if !sessions_are_consistent(&currents) {
            return false;
        }
        let mut index = self.index.clone();
        let mut stager = self.stager.clone();
        if !index.install_current_snapshot(currents.iter().map(|current| {
            (
                current.revision().section_key.clone(),
                current.revision().revision,
            )
        })) {
            return false;
        }
        let Some(revisions) = hydrate_currents(&mut index, &currents) else {
            return false;
        };
        if !stager.restore_hydrated_fence(&revisions) {
            return false;
        }
        self.index = index;
        self.stager = stager;
        true
    }
}

fn hydrate_currents(
    index: &mut observation_ingest::DecisionRevisionIndex,
    currents: &[CurrentRevision],
) -> Option<Vec<observation_ingest::HydratedSectionRevision>> {
    currents
        .iter()
        .map(|current| {
            let revision = current.hydrate().ok()?;
            index
                .restore_committed(revision.clone(), current.receipt().accepted_at)
                .then_some(revision)
        })
        .collect()
}

fn sessions_are_consistent(currents: &[CurrentRevision]) -> bool {
    let mut sessions: BTreeMap<&SourceScopeId, &SourceSessionIdentity> = BTreeMap::new();
    currents.iter().all(|current| {
        let revision = current.revision();
        sessions
            .insert(&revision.source_scope, &revision.source_session)
            .is_none_or(|existing| existing == &revision.source_session)
    })
}
