use super::*;

const fn matching_terminal(coverage: SectionCoverage) -> CompletionCoverage {
    match coverage {
        SectionCoverage::Complete => CompletionCoverage::Complete,
        SectionCoverage::KnownEmpty => CompletionCoverage::KnownEmpty,
        SectionCoverage::Partial => CompletionCoverage::Partial,
        SectionCoverage::Unknown => CompletionCoverage::Unknown,
        SectionCoverage::Unsupported => CompletionCoverage::Unsupported,
    }
}

fn empty_candidate(coverage: SectionCoverage) -> GenerationStager {
    let mut stager = GenerationStager::new(AcceptedProjection::empty(), limits());
    let mut start = start();
    start.expected_records = 0;
    let context = CandidateContext::new(
        versions(),
        CaptureWindow::new(10, 20).expect("window is ordered"),
        SectionState::with_evidence(
            CaptureWindow::new(10, 20).expect("window is ordered"),
            SectionFreshness::Fresh,
            SectionQuality::Fresh,
            SectionAvailability::Available,
            coverage,
        ),
        BTreeMap::from([(key("sectors"), revision(4))]),
        Some(revision(6)),
        true,
    );
    assert_eq!(
        stager.start_section_with_context(start, context, 1),
        ReceiverDisposition::Received
    );
    stager
}

#[test]
fn terminal_coverage_must_exactly_match_frozen_source_evidence() {
    let coverages = [
        SectionCoverage::Complete,
        SectionCoverage::KnownEmpty,
        SectionCoverage::Partial,
        SectionCoverage::Unknown,
        SectionCoverage::Unsupported,
    ];
    for source in coverages {
        for terminal_source in coverages {
            let mut stager = empty_candidate(source);
            let mut envelope = completion();
            envelope.record_count = 0;
            envelope.coverage = matching_terminal(terminal_source);
            let certificate = stager
                .completion_certificate(envelope)
                .expect("candidate exists");
            let outcome = stager.complete_section(&certificate, &current(), 2);
            assert_eq!(
                matches!(outcome, CompletionOutcome::Validated(_)),
                source == terminal_source
            );
        }
    }
}

#[test]
fn known_empty_rejects_non_empty_content_even_with_stable_identity() {
    let mut stager = staged();
    let mut envelope = completion();
    envelope.coverage = CompletionCoverage::KnownEmpty;
    let certificate = stager
        .completion_certificate(envelope)
        .expect("candidate exists");
    assert_eq!(
        stager.complete_section(&certificate, &current(), 4),
        CompletionOutcome::Rejected(RejectionReason::CompletionMismatch)
    );
}
