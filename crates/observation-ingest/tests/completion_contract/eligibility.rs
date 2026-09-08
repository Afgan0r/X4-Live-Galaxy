use super::*;

#[test]
fn eligible_revision_becomes_stale_then_history_only_under_uncertainty() {
    let mut index = DecisionRevisionIndex::new(4).expect("blocker limit is non-zero");
    index.record_current_pointer(key("ships"), revision(6));
    index.record_current_pointer(key("sectors"), revision(4));
    let accepted = index
        .accept(finish(&mut staged()), 4)
        .expect("current session is accepted");
    assert_eq!(
        index.finalize_committed(&accepted, 4),
        FinalizationOutcome::Finalized
    );
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
        .prepare_publication(finish(&mut staged()))
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
        .prepare_publication(finish(&mut staged()))
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

#[test]
fn publication_rotates_only_to_a_higher_epoch_of_the_same_producer() {
    let mut index = DecisionRevisionIndex::new(4).expect("blocker limit is non-zero");
    index.record_current_pointer(key("ships"), revision(6));
    index.record_current_pointer(key("sectors"), revision(4));
    let first = finish(&mut staged());
    let accepted = index
        .prepare_publication(first)
        .expect("first source session prepares");
    let higher_epoch = finish_with_session("producer:1", 2);
    let replacement = index
        .prepare_publication(higher_epoch)
        .expect("higher epoch rotates authority before finalization");
    assert_eq!(
        index.finalize_committed(&accepted, 4),
        FinalizationOutcome::AuthorityChanged
    );
    assert_eq!(
        index.finalize_committed(&replacement, 5),
        FinalizationOutcome::Finalized
    );
    assert!(
        index
            .prepare_publication(finish_with_session("producer:1", 1))
            .is_none()
    );
    assert!(
        index
            .prepare_publication(finish_with_session("producer:2", 3))
            .is_none()
    );
}

#[test]
fn finalization_checks_pointer_and_dependencies_independently() {
    for (ships, sectors) in [(revision(99), revision(4)), (revision(6), revision(99))] {
        let mut index = DecisionRevisionIndex::new(4).expect("blocker limit");
        index.record_current_pointer(key("ships"), ships);
        index.record_current_pointer(key("sectors"), sectors);
        let accepted = index
            .prepare_publication(finish(&mut staged()))
            .expect("publication prepares before concurrent pointer change");
        assert_eq!(
            index.finalize_committed(&accepted, 4),
            FinalizationOutcome::StateMismatch
        );
    }
}

fn finish_with_session(producer: &str, epoch: u64) -> observation_ingest::ValidatedSectionRevision {
    let incarnation = || value(producer, ProducerIncarnationId::new);
    let transport_epoch = || TransportEpoch::new(epoch).expect("epoch is positive");
    let mut stager = GenerationStager::new(AcceptedProjection::empty(), limits());
    let mut start = start();
    start.producer_incarnation = incarnation();
    start.transport_epoch = transport_epoch();
    assert_eq!(
        stager.start_section_with_context(start, context(), 1),
        ReceiverDisposition::Received
    );
    let mut batches = [
        batch("batch:1", 1, "ship:2", "record:2"),
        batch("batch:2", 2, "ship:1", "record:1"),
    ];
    for (now, batch) in [2_u64, 3].into_iter().zip(&mut batches) {
        batch.producer_incarnation = incarnation();
        batch.transport_epoch = transport_epoch();
        assert_eq!(
            stager.stage_section_batch(batch.clone(), 1, now),
            ReceiverDisposition::Received
        );
    }
    let mut completion = completion();
    completion.producer_incarnation = incarnation();
    completion.transport_epoch = transport_epoch();
    let envelope =
        observation_ingest::bind_completion_certificate(completion, &batches, versions())
            .expect("producer certificate binds");
    let certificate = stager
        .completion_certificate(envelope)
        .expect("candidate exists");
    let revision = match stager.complete_section(&certificate, &current(), 4) {
        CompletionOutcome::Validated(revision) => Some(revision),
        CompletionOutcome::Rejected(_) => None,
    }
    .expect("exact completion validates");
    *revision
}
