use std::cell::Cell;
use std::rc::Rc;

use observation_domain::{SectionKey, SectionRevisionId, SourceScopeId, SourceSessionIdentity};

use super::{DecisionRevisionIndex, SessionAuthority};
use crate::ValidatedSectionRevision;

#[must_use]
#[derive(Clone, Debug)]
pub struct AcceptedPublication {
    revision: ValidatedSectionRevision,
    authority_generation: Rc<Cell<u64>>,
    accepted_generation: u64,
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum FinalizationOutcome {
    Finalized,
    AlreadyFinalized,
    AuthorityChanged,
    StateMismatch,
}

impl AcceptedPublication {
    pub const fn revision(&self) -> &ValidatedSectionRevision {
        &self.revision
    }

    #[must_use]
    pub fn is_authoritative(&self) -> bool {
        self.authority_generation.get() == self.accepted_generation
    }
}

impl SessionAuthority {
    pub(super) fn new(identity: SourceSessionIdentity) -> Self {
        Self {
            identity,
            generation: Rc::new(Cell::new(0)),
        }
    }

    fn accepted(&self, revision: ValidatedSectionRevision) -> AcceptedPublication {
        AcceptedPublication {
            revision,
            authority_generation: Rc::clone(&self.generation),
            accepted_generation: self.generation.get(),
        }
    }

    fn replace(&mut self, identity: SourceSessionIdentity) {
        self.identity = identity;
        self.generation.set(self.generation.get().saturating_add(1));
    }
}

impl DecisionRevisionIndex {
    pub fn prepare_publication(
        &mut self,
        revision: ValidatedSectionRevision,
    ) -> Option<AcceptedPublication> {
        let scope = revision.source_scope().clone();
        if self
            .authoritative_sessions
            .get(&scope)
            .is_some_and(|current| &current.identity != revision.source_session())
        {
            return None;
        }
        self.authoritative_sessions
            .entry(scope.clone())
            .or_insert_with(|| SessionAuthority::new(revision.source_session().clone()));
        self.authoritative_sessions
            .get(&scope)
            .map(|authority| authority.accepted(revision))
    }

    pub fn accept(
        &mut self,
        revision: ValidatedSectionRevision,
        _accepted_at: u64,
    ) -> Option<AcceptedPublication> {
        self.prepare_publication(revision)
    }

    pub fn finalize_committed(
        &mut self,
        accepted: &AcceptedPublication,
        accepted_at: u64,
    ) -> FinalizationOutcome {
        let revision = accepted.revision();
        let scope = revision.source_scope();
        if self
            .authoritative_sessions
            .get(scope)
            .is_none_or(|current| &current.identity != revision.source_session())
            || !accepted.is_authoritative()
        {
            return FinalizationOutcome::AuthorityChanged;
        }
        let key = revision.section_key();
        if self
            .current
            .get(key)
            .is_some_and(|(current, _)| current == revision)
        {
            return FinalizationOutcome::AlreadyFinalized;
        }
        let pointer_matches =
            self.pointers.get(key).copied() == revision.context().expected_current();
        let dependencies_match = revision
            .context()
            .dependencies()
            .iter()
            .all(|(dependency, expected)| self.pointers.get(dependency) == Some(expected));
        if !pointer_matches || !dependencies_match {
            return FinalizationOutcome::StateMismatch;
        }
        self.uncertain_scopes.remove(scope);
        self.pointers
            .insert(key.clone(), revision.section_revision());
        if let Some(previous) = self
            .current
            .insert(key.clone(), (revision.clone(), accepted_at))
        {
            self.history.push(previous);
        }
        FinalizationOutcome::Finalized
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
            .entry(scope.clone())
            .and_modify(|authority| authority.replace(authoritative_session.clone()))
            .or_insert_with(|| SessionAuthority::new(authoritative_session));
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
