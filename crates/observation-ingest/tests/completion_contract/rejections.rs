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
