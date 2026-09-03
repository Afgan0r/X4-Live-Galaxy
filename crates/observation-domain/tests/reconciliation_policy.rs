#![expect(
    clippy::expect_used,
    reason = "test fixtures fail immediately when their invariants are invalid"
)]

use observation_domain::{
    AbsenceEvidence, AbsenceTracker, CanonicalObservationKey, CollectionLimit, CollectionSize,
    CompleteMarker, CompletionCoverage, CountError, EntityId, ObservationVersion,
    ReconciliationDecision, SourceScopeId, reconcile_membership, reconcile_qualified_membership,
};

fn entity(value: &str) -> EntityId {
    EntityId::new(value).expect("test identity is valid")
}

fn source(value: &str) -> SourceScopeId {
    SourceScopeId::new(value).expect("test source scope is valid")
}

const fn version(value: u64) -> ObservationVersion {
    ObservationVersion::new(value).expect("test version is positive")
}

fn key(value: &str, key_version: u64) -> CanonicalObservationKey {
    CanonicalObservationKey::new(entity(value), version(key_version))
}

#[test]
fn complete_scope_reconciles_sorted_members_and_tombstones() {
    let scope = entity("scope:sectors");
    let prior = vec![key("sector:alpha", 1), key("sector:gamma", 1)];
    let observed = vec![key("sector:beta", 2), key("sector:alpha", 2)];

    assert_eq!(
        reconcile_membership(
            &prior,
            observed,
            &scope,
            Some(&CompleteMarker::successful(scope.clone(), version(2))),
            CollectionLimit::new(2).expect("positive limit"),
        ),
        ReconciliationDecision::Reconciled {
            members: vec![key("sector:alpha", 2), key("sector:beta", 2)],
            tombstones: vec![key("sector:gamma", 1)],
        }
    );
}

#[test]
fn incomplete_or_other_scope_preserves_previous_membership() {
    let scope = entity("scope:sectors");
    let prior = vec![key("sector:alpha", 1)];

    for marker in [
        None,
        Some(CompleteMarker::successful(
            entity("scope:assets"),
            version(1),
        )),
    ] {
        assert_eq!(
            reconcile_membership(
                &prior,
                vec![],
                &scope,
                marker.as_ref(),
                CollectionLimit::new(1).expect("positive limit"),
            ),
            ReconciliationDecision::PreservedIncompleteScope
        );
    }
}

#[test]
fn explicit_collection_boundaries_reject_overflow_without_precision_loss() {
    let scope = entity("scope:sectors");
    let limit = CollectionLimit::new(1).expect("positive limit");

    assert_eq!(
        reconcile_membership(
            &[],
            vec![key("sector:alpha", 1)],
            &scope,
            Some(&CompleteMarker::successful(scope.clone(), version(1))),
            limit,
        ),
        ReconciliationDecision::Reconciled {
            members: vec![key("sector:alpha", 1)],
            tombstones: vec![],
        }
    );
    assert_eq!(
        reconcile_membership(
            &[],
            vec![key("sector:alpha", 1), key("sector:beta", 1)],
            &scope,
            Some(&CompleteMarker::successful(scope.clone(), version(1))),
            limit,
        ),
        ReconciliationDecision::RejectedCollectionLimit
    );
    assert_eq!(
        CollectionSize::from_u128((usize::MAX as u128) + 1),
        Err(CountError::ExceedsPlatformCapacity)
    );
}

#[test]
fn deletion_requires_two_qualifying_same_scope_absences() {
    let prior = vec![key("ship:alpha", 1)];
    let mut tracker = AbsenceTracker::new();
    let evidence = AbsenceEvidence::new(
        source("scope:x4"),
        CompletionCoverage::Complete,
        true,
        true,
        false,
    );
    let limit = CollectionLimit::new(1).expect("positive limit");

    assert_eq!(
        reconcile_qualified_membership(&prior, vec![], &evidence, &mut tracker, limit),
        ReconciliationDecision::AwaitingSecondAbsence {
            members: prior.clone()
        }
    );
    assert_eq!(
        reconcile_qualified_membership(&prior, vec![], &evidence, &mut tracker, limit),
        ReconciliationDecision::Reconciled {
            members: vec![],
            tombstones: prior
        }
    );
}

#[test]
fn weak_evidence_and_scope_change_have_no_absence_meaning() {
    let prior = vec![key("ship:alpha", 1)];
    let mut tracker = AbsenceTracker::new();
    let limit = CollectionLimit::new(1).expect("positive limit");
    for evidence in [
        AbsenceEvidence::new(
            source("scope:x4"),
            CompletionCoverage::Partial,
            true,
            true,
            false,
        ),
        AbsenceEvidence::new(
            source("scope:x4"),
            CompletionCoverage::Complete,
            false,
            true,
            false,
        ),
        AbsenceEvidence::new(
            source("scope:x4"),
            CompletionCoverage::Complete,
            true,
            false,
            true,
        ),
    ] {
        assert_eq!(
            reconcile_qualified_membership(&prior, vec![], &evidence, &mut tracker, limit),
            ReconciliationDecision::PreservedIncompleteScope
        );
    }
    let first = AbsenceEvidence::new(
        source("scope:a"),
        CompletionCoverage::Complete,
        true,
        true,
        false,
    );
    let changed = AbsenceEvidence::new(
        source("scope:b"),
        CompletionCoverage::Complete,
        true,
        true,
        false,
    );
    let _ = reconcile_qualified_membership(&prior, vec![], &first, &mut tracker, limit);
    assert!(matches!(
        reconcile_qualified_membership(&prior, vec![], &changed, &mut tracker, limit),
        ReconciliationDecision::AwaitingSecondAbsence { .. }
    ));
}
