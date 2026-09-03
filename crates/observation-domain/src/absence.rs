use std::collections::BTreeMap;

use crate::{
    CanonicalObservationKey, CollectionLimit, CompletionCoverage, EntityId, ReconciliationDecision,
    SourceScopeId,
};

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct AbsenceEvidence {
    source_scope: SourceScopeId,
    coverage: CompletionCoverage,
    stable_identity: bool,
    core_complete: bool,
    _optional_failure: bool,
}

impl AbsenceEvidence {
    pub const fn new(
        source_scope: SourceScopeId,
        coverage: CompletionCoverage,
        stable_identity: bool,
        core_complete: bool,
        optional_failure: bool,
    ) -> Self {
        Self {
            source_scope,
            coverage,
            stable_identity,
            core_complete,
            _optional_failure: optional_failure,
        }
    }
}

#[must_use]
#[derive(Clone, Debug, Default, Eq, PartialEq)]
pub struct AbsenceTracker {
    source_scope: Option<SourceScopeId>,
    streaks: BTreeMap<EntityId, u8>,
}

impl AbsenceTracker {
    pub const fn new() -> Self {
        Self {
            source_scope: None,
            streaks: BTreeMap::new(),
        }
    }
}

pub fn reconcile_qualified_membership(
    previous: &[CanonicalObservationKey],
    mut observed: Vec<CanonicalObservationKey>,
    evidence: &AbsenceEvidence,
    tracker: &mut AbsenceTracker,
    limit: CollectionLimit,
) -> ReconciliationDecision {
    if observed.len() > limit.get() {
        return ReconciliationDecision::RejectedCollectionLimit;
    }
    let coverage_qualifies = matches!(evidence.coverage, CompletionCoverage::Complete)
        || (matches!(evidence.coverage, CompletionCoverage::KnownEmpty) && observed.is_empty());
    if !coverage_qualifies || !evidence.stable_identity || !evidence.core_complete {
        tracker.streaks.clear();
        return ReconciliationDecision::PreservedIncompleteScope;
    }
    if tracker.source_scope.as_ref() != Some(&evidence.source_scope) {
        tracker.source_scope = Some(evidence.source_scope.clone());
        tracker.streaks.clear();
    }
    observed.sort();
    observed.dedup();
    reconcile_absences(previous, observed, tracker)
}

fn reconcile_absences(
    previous: &[CanonicalObservationKey],
    observed: Vec<CanonicalObservationKey>,
    tracker: &mut AbsenceTracker,
) -> ReconciliationDecision {
    let mut tombstones = Vec::new();
    let mut awaiting = false;
    for prior in previous {
        if observed
            .iter()
            .any(|current| current.entity_id() == prior.entity_id())
        {
            tracker.streaks.remove(prior.entity_id());
            continue;
        }
        let streak = tracker
            .streaks
            .entry(prior.entity_id().clone())
            .or_insert(0);
        *streak = streak.saturating_add(1).min(2);
        if *streak == 2 {
            tombstones.push(prior.clone());
        } else {
            awaiting = true;
        }
    }
    if awaiting {
        let mut members = previous.to_vec();
        members.extend(observed);
        members.sort();
        members.dedup();
        return ReconciliationDecision::AwaitingSecondAbsence { members };
    }
    tombstones.sort();
    ReconciliationDecision::Reconciled {
        members: observed,
        tombstones,
    }
}
