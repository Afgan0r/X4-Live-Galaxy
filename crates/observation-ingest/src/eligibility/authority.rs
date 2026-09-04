use observation_domain::{SectionKey, SectionRevisionId, SourceScopeId, SourceSessionIdentity};

use super::DecisionRevisionIndex;
use crate::ValidatedSectionRevision;

impl DecisionRevisionIndex {
    pub fn accept(&mut self, revision: ValidatedSectionRevision, accepted_at: u64) -> bool {
        let scope = revision.source_scope().clone();
        if self
            .authoritative_sessions
            .get(&scope)
            .is_some_and(|current| current != revision.source_session())
        {
            self.history.push((revision, accepted_at));
            return false;
        }
        let key = revision.section_key().clone();
        self.authoritative_sessions
            .entry(scope.clone())
            .or_insert_with(|| revision.source_session().clone());
        self.uncertain_scopes.remove(&scope);
        self.pointers
            .insert(key.clone(), revision.section_revision());
        if let Some(previous) = self.current.insert(key, (revision, accepted_at)) {
            self.history.push(previous);
        }
        true
    }

    pub fn record_current_pointer(&mut self, key: SectionKey, revision: SectionRevisionId) {
        self.pointers.insert(key, revision);
    }

    pub fn mark_scope_uncertain(
        &mut self,
        scope: &SourceScopeId,
        authoritative_session: SourceSessionIdentity,
    ) {
        self.uncertain_scopes.insert(scope.clone());
        self.authoritative_sessions
            .insert(scope.clone(), authoritative_session);
        let keys: Vec<_> = self
            .current
            .iter()
            .filter(|(_, (revision, _))| revision.source_scope() == scope)
            .map(|(key, _)| key.clone())
            .collect();
        for key in keys {
            self.current
                .remove(&key)
                .into_iter()
                .for_each(|revision| self.history.push(revision));
            self.pointers.remove(&key);
        }
    }

    #[must_use]
    pub fn current_count(&self) -> usize {
        self.current.len()
    }

    #[must_use]
    pub const fn history_count(&self) -> usize {
        self.history.len()
    }
}
