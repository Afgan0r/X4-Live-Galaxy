use super::*;

#[test]
fn any_count_length_digest_or_version_mismatch_discards_candidate() {
    let mutators: [fn(&mut observation_ingest::CompletionCertificate); 10] = [
        |value| value.envelope.batch_count += 1,
        |value| value.envelope.record_count += 1,
        |value| value.envelope.raw_bytes += 1,
        |value| value.envelope.decoded_bytes += 1,
        |value| value.envelope.ordered_batch_manifest_digest[0] ^= 1,
        |value| value.envelope.canonical_content_digest[0] ^= 1,
        |value| {
            value.envelope.schema_version =
                ObservationSchemaVersion::new(9).expect("version is positive");
        },
        |value| {
            value.envelope.policy_version =
                ObservationPolicyVersion::new(9).expect("version is positive");
        },
        |value| {
            value.envelope.canonicalization_version =
                CanonicalizationVersion::new(9).expect("version is positive");
        },
        |value| {
            value.envelope.digest_version =
                DigestAlgorithmVersion::new(9).expect("version is positive");
        },
    ];
    for mutate in mutators {
        let mut stager = staged();
        let mut certificate = exact_certificate(&stager);
        mutate(&mut certificate);
        assert_eq!(
            stager.complete_section(&certificate, &current(), 4),
            CompletionOutcome::Rejected(RejectionReason::CompletionMismatch)
        );
        assert_eq!(stager.candidate_count(), 0);
        assert_eq!(stager.aggregate_usage().candidate_count, 0);
    }
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
        let certificate = exact_certificate(&stager);
        assert_eq!(
            stager.complete_section(&certificate, &current(), 4),
            CompletionOutcome::Rejected(RejectionReason::CompletionMismatch)
        );
        assert_eq!(stager.candidate_count(), 0);
        assert_eq!(stager.aggregate_usage().candidate_count, 0);
    }
}
