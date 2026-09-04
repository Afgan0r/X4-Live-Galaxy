use super::*;

#[test]
fn eligible_revision_becomes_stale_then_history_only_under_uncertainty() {
    let mut index = DecisionRevisionIndex::new(4).expect("blocker limit is non-zero");
    let _accepted = index
        .accept(finish(&mut staged()), 4)
        .expect("current session is accepted");
    index.record_current_pointer(key("sectors"), revision(4));
    let set = match index.eligibility(&[key("ships")], 10, 10) {
        DecisionEligibility::Eligible(set) => Some(set),
        DecisionEligibility::Blocked(_) => None,
    }
    .expect("fresh exact revision is eligible");
    assert_eq!(set.revisions().len(), 1);
    assert!(set.revisions().contains_key(&key("ships")));
    assert_eq!(
        index.eligibility(&[key("ships")], 15, 10),
        DecisionEligibility::Blocked(vec![EligibilityBlocker::Stale(key("ships"))])
    );
    index.mark_scope_uncertain(
        &value("scope:x4", SourceScopeId::new),
        SourceSessionIdentity::new(
            value("producer:2", ProducerIncarnationId::new),
            TransportEpoch::new(2).expect("epoch is positive"),
        ),
    );
    assert_eq!(index.current_count(), 0);
    assert_eq!(index.history_count(), 1);
    assert!(matches!(
        index.eligibility(&[key("ships")], 5, 10),
        DecisionEligibility::Blocked(ref blockers)
            if blockers == &[EligibilityBlocker::Uncertain(value("scope:x4", SourceScopeId::new))]
    ));
}

#[test]
fn preparation_is_non_mutating_and_finalization_requires_live_authority() {
    let mut index = DecisionRevisionIndex::new(4).expect("blocker limit is non-zero");
    index.record_current_pointer(key("ships"), revision(6));
    index.record_current_pointer(key("sectors"), revision(4));
    let accepted = index
        .prepare_publication(finish(&mut staged()), 4)
        .expect("current session prepares");

    assert_eq!(index.current_count(), 0);
    assert_eq!(index.history_count(), 0);
    assert!(matches!(
        index.eligibility(&[key("ships")], 4, 10),
        DecisionEligibility::Blocked(_)
    ));
    assert_eq!(
        index.finalize_committed(&accepted, 4),
        FinalizationOutcome::Finalized
    );
    assert_eq!(
        index.finalize_committed(&accepted, 4),
        FinalizationOutcome::AlreadyFinalized
    );
    assert_eq!(index.current_count(), 1);

    let pending = index
        .prepare_publication(finish(&mut staged()), 5)
        .expect("same session prepares");
    index.mark_scope_uncertain(
        &value("scope:x4", SourceScopeId::new),
        SourceSessionIdentity::new(
            value("producer:2", ProducerIncarnationId::new),
            TransportEpoch::new(2).expect("epoch is positive"),
        ),
    );
    assert_eq!(
        index.finalize_committed(&pending, 5),
        FinalizationOutcome::AuthorityChanged
    );
    assert_eq!(index.current_count(), 0);
}
