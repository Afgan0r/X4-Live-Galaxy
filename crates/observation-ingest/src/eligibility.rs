use std::collections::{BTreeMap, BTreeSet};
use std::num::NonZeroUsize;

use observation_domain::{
    CompletionCoverage, SectionAvailability, SectionFreshness, SectionKey, SectionQuality,
    SectionRevisionId, SourceScopeId,
};

use crate::ValidatedSectionRevision;

#[must_use]
#[derive(Clone, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum EligibilityBlocker {
    Uncertain(SourceScopeId),
    Missing(SectionKey),
    Stale(SectionKey),
    EvidenceInsufficient(SectionKey),
    DependencyMismatch(SectionKey),
    ScopeMismatch(SectionKey),
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct DecisionRevisionSet {
    revisions: BTreeMap<SectionKey, SectionRevisionId>,
}

impl DecisionRevisionSet {
    #[must_use]
    pub const fn revisions(&self) -> &BTreeMap<SectionKey, SectionRevisionId> {
        &self.revisions
    }
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum DecisionEligibility {
    Eligible(DecisionRevisionSet),
    Blocked(Vec<EligibilityBlocker>),
}

pub struct DecisionRevisionIndex {
    blocker_limit: NonZeroUsize,
    current: BTreeMap<SectionKey, (ValidatedSectionRevision, u64)>,
    pointers: BTreeMap<SectionKey, SectionRevisionId>,
    history: Vec<(ValidatedSectionRevision, u64)>,
    uncertain_scopes: BTreeSet<SourceScopeId>,
}

impl DecisionRevisionIndex {
    #[must_use]
    pub fn new(blocker_limit: usize) -> Option<Self> {
        Some(Self {
            blocker_limit: NonZeroUsize::new(blocker_limit)?,
            current: BTreeMap::new(),
            pointers: BTreeMap::new(),
            history: Vec::new(),
            uncertain_scopes: BTreeSet::new(),
        })
    }
    pub fn accept(&mut self, revision: ValidatedSectionRevision, accepted_at: u64) {
        let key = revision.section_key().clone();
        self.uncertain_scopes.remove(revision.source_scope());
        self.pointers
            .insert(key.clone(), revision.section_revision());
        if let Some(previous) = self.current.insert(key, (revision, accepted_at)) {
            self.history.push(previous);
        }
    }
    pub fn record_current_pointer(&mut self, key: SectionKey, revision: SectionRevisionId) {
        self.pointers.insert(key, revision);
    }
    pub fn mark_scope_uncertain(&mut self, scope: &SourceScopeId) {
        self.uncertain_scopes.insert(scope.clone());
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

    pub fn eligibility(
        &self,
        required: &[SectionKey],
        now: u64,
        max_age: u64,
    ) -> DecisionEligibility {
        let mut required = required.to_vec();
        required.sort();
        required.dedup();
        let mut blockers = Vec::new();
        let mut revisions = BTreeMap::new();
        let mut scope = None;
        for key in required {
            scope = self.evaluate_required(key, now, max_age, scope, &mut blockers, &mut revisions);
        }
        blockers.sort();
        blockers.dedup();
        blockers.truncate(self.blocker_limit.get());
        if blockers.is_empty() {
            DecisionEligibility::Eligible(DecisionRevisionSet { revisions })
        } else {
            DecisionEligibility::Blocked(blockers)
        }
    }

    fn evaluate_required<'a>(
        &'a self,
        key: SectionKey,
        now: u64,
        max_age: u64,
        scope: Option<&'a SourceScopeId>,
        blockers: &mut Vec<EligibilityBlocker>,
        revisions: &mut BTreeMap<SectionKey, SectionRevisionId>,
    ) -> Option<&'a SourceScopeId> {
        let Some((revision, accepted_at)) = self.current.get(&key) else {
            blockers.push(self.missing_blocker(key));
            return scope;
        };
        blockers.extend(self.revision_blockers(&key, revision, *accepted_at, now, max_age, scope));
        revisions.insert(key, revision.section_revision());
        Some(revision.source_scope())
    }

    fn missing_blocker(&self, key: SectionKey) -> EligibilityBlocker {
        self.history
            .iter()
            .rev()
            .find(|(item, _)| item.section_key() == &key)
            .map(|(item, _)| item.source_scope())
            .filter(|scope| self.uncertain_scopes.contains(*scope))
            .map_or(EligibilityBlocker::Missing(key), |scope| {
                EligibilityBlocker::Uncertain(scope.clone())
            })
    }

    fn revision_blockers(
        &self,
        key: &SectionKey,
        revision: &ValidatedSectionRevision,
        accepted_at: u64,
        now: u64,
        max_age: u64,
        scope: Option<&SourceScopeId>,
    ) -> Vec<EligibilityBlocker> {
        let mut blockers = Vec::new();
        if now.saturating_sub(accepted_at) > max_age {
            blockers.push(EligibilityBlocker::Stale(key.clone()));
        }
        if !evidence_qualifies(revision) {
            blockers.push(EligibilityBlocker::EvidenceInsufficient(key.clone()));
        }
        let dependencies_match = revision
            .context()
            .dependencies()
            .iter()
            .all(|(dependency, expected)| self.pointers.get(dependency) == Some(expected));
        if !dependencies_match {
            blockers.push(EligibilityBlocker::DependencyMismatch(key.clone()));
        }
        if scope.is_some_and(|value| value != revision.source_scope()) {
            blockers.push(EligibilityBlocker::ScopeMismatch(key.clone()));
        }
        blockers
    }
}

fn evidence_qualifies(revision: &ValidatedSectionRevision) -> bool {
    let state = revision.context().state();
    revision.context().stable_identity()
        && state.freshness() == SectionFreshness::Fresh
        && state.availability() == SectionAvailability::Available
        && matches!(
            revision.coverage(),
            CompletionCoverage::Complete | CompletionCoverage::KnownEmpty
        )
        && !matches!(
            state.quality(),
            SectionQuality::Unknown
                | SectionQuality::Partial
                | SectionQuality::Stale
                | SectionQuality::Unsupported
        )
}
