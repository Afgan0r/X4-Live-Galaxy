use super::*;

#[test]
fn any_count_length_digest_or_version_mismatch_discards_candidate() {
    let mut stager = staged();
    let mut certificate = stager
        .completion_certificate(completion())
        .expect("candidate exists");
    certificate.record_count += 1;
    assert_eq!(
        stager.complete_section(&certificate, &current(), 4),
        CompletionOutcome::Rejected(RejectionReason::CompletionMismatch)
    );
    assert_eq!(stager.candidate_count(), 0);
    assert_eq!(stager.aggregate_usage().candidate_count, 0);
}

#[test]
fn completion_rejects_entity_versions_superseded_after_staging() {
    for (accepted_version, accepted_content) in [
        (2, b"accepted:new".as_slice()),
        (1, b"accepted:conflict".as_slice()),
    ] {
        let mut stager = staged();
        assert!(stager.record_accepted_entity(
            value("scope:x4", SourceScopeId::new),
            value("ship:1", EntityId::new),
            ObservationVersion::new(accepted_version).expect("version is positive"),
            accepted_content,
        ));
        let certificate = stager
            .completion_certificate(completion())
            .expect("candidate exists");
        assert_eq!(
            stager.complete_section(&certificate, &current(), 4),
            CompletionOutcome::Rejected(RejectionReason::CompletionMismatch)
        );
        assert_eq!(stager.candidate_count(), 0);
        assert_eq!(stager.aggregate_usage().candidate_count, 0);
    }
}
