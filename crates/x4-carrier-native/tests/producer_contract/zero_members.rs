use observation_domain::{
    CaptureWindow, SectionAvailability, SectionCoverage, SectionFreshness, SectionQuality,
    SectionState, SourceConsistency,
};
use x4_carrier_native::{ProducerError, SectionEvidence};

use super::ship_support::ready_ship;

#[test]
fn zero_records_preserve_non_authoritative_observation_evidence() {
    let (mut producer, _) = ready_ship(0);
    let window = CaptureWindow::new(0, 0).expect("window is ordered");
    let mut partial = SectionEvidence::point_measurement("x4:faction:argon:ships");
    partial.sender.section_state = SectionState::with_evidence(
        window,
        SectionFreshness::Fresh,
        SectionQuality::Unknown,
        SectionAvailability::Available,
        SectionCoverage::Partial,
    );
    partial.sender.source_consistency = SourceConsistency::ObservedCountFillOnly;
    partial.sender.stable_identity = true;
    assert_eq!(
        producer.begin_ship_section(partial, 0),
        Ok(()),
        "zero members are observed without claiming authoritative empty"
    );
    let (mut producer, _) = ready_ship(0);
    let mut evidence = SectionEvidence::point_measurement("x4:faction:argon:ships");
    evidence.sender.section_state = SectionState::with_evidence(
        window,
        SectionFreshness::Fresh,
        SectionQuality::KnownEmpty,
        SectionAvailability::Available,
        SectionCoverage::KnownEmpty,
    );
    evidence.sender.source_consistency = SourceConsistency::Barrier;
    assert_eq!(producer.begin_ship_section(evidence, 0), Ok(()));
}

#[test]
fn zero_records_reject_each_missing_authority_condition() {
    let window = CaptureWindow::new(0, 0).expect("window is ordered");
    for (coverage, consistency, stable_identity) in [
        (
            SectionCoverage::KnownEmpty,
            SourceConsistency::ObservedCountFillOnly,
            true,
        ),
        (SectionCoverage::Partial, SourceConsistency::Barrier, true),
        (
            SectionCoverage::Partial,
            SourceConsistency::ObservedCountFillOnly,
            false,
        ),
    ] {
        let (mut producer, _) = ready_ship(0);
        let mut evidence = SectionEvidence::point_measurement("x4:faction:argon:ships");
        evidence.sender.section_state = SectionState::with_evidence(
            window,
            SectionFreshness::Fresh,
            SectionQuality::Unknown,
            SectionAvailability::Available,
            coverage,
        );
        evidence.sender.source_consistency = consistency;
        evidence.sender.stable_identity = stable_identity;
        assert_eq!(
            producer.begin_ship_section(evidence, 0),
            Err(ProducerError::InvalidInput),
            "zero members require complete authority: {coverage:?} {consistency:?} {stable_identity}"
        );
    }
}
