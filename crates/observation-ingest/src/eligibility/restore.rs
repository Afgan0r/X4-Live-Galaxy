use std::collections::BTreeMap;

use observation_domain::{SectionKey, SectionRevisionId};

use super::{DecisionRevisionIndex, SessionAuthority};
use crate::HydratedSectionRevision;

impl DecisionRevisionIndex {
    pub fn install_current_snapshot(
        &mut self,
        pointers: impl IntoIterator<Item = (SectionKey, SectionRevisionId)>,
    ) -> bool {
        if !self.current.is_empty() || !self.history.is_empty() {
            return false;
        }
        let entries: Vec<_> = pointers.into_iter().collect();
        let installed: BTreeMap<_, _> = entries.iter().cloned().collect();
        if installed.len() != entries.len() {
            return false;
        }
        self.pointers = installed;
        true
    }

    pub fn restore_committed(
        &mut self,
        revision: HydratedSectionRevision,
        accepted_at: u64,
    ) -> bool {
        if self.pointers.get(revision.section_key()) != Some(&revision.section_revision()) {
            return false;
        }
        let scope = revision.source_scope().clone();
        if self
            .authoritative_sessions
            .get(&scope)
            .is_some_and(|current| &current.identity != revision.source_session())
        {
            return false;
        }
        self.authoritative_sessions
            .entry(scope.clone())
            .or_insert_with(|| SessionAuthority::new(revision.source_session().clone()));
        self.uncertain_scopes.remove(&scope);
        self.current
            .insert(
                revision.section_key().clone(),
                (revision.into_validated(), accepted_at),
            )
            .is_none()
    }
}
