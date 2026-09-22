use crate::ship_support::{core_messages, ready, take_messages};
use observation_domain::*;
use x4_carrier_native::{SectionEvidence, SectionFinishEvidence};

pub fn scoped_messages(revision: u64, faction: &str) -> Vec<Vec<u8>> {
    core_messages(
        revision,
        faction,
        faction,
        ["9007199254740993", "9007199254740995"],
        None,
        &format!("ship_core:{faction}"),
    )
}

pub fn scoped_zero_messages(revision: u64, faction: &str) -> Vec<Vec<u8>> {
    let key = format!("ship_core:{faction}");
    let (mut producer, source) = ready(revision, faction, &key);
    let mut evidence = SectionEvidence::point_measurement(source.source_scope.clone());
    evidence.sender.section_state = SectionState::with_evidence(
        CaptureWindow::new(0, 0).expect("valid test fixture"),
        SectionFreshness::Fresh,
        SectionQuality::Unknown,
        SectionAvailability::Available,
        SectionCoverage::Partial,
    );
    evidence.sender.source_consistency = SourceConsistency::ObservedCountFillOnly;
    evidence.sender.stable_identity = true;
    producer
        .begin_ship_section(evidence, 0)
        .expect("zero-member observation is valid");
    producer
        .finish_section(SectionFinishEvidence {
            capture_end_millis: 0,
            succeeded: true,
            quality: SectionQuality::Unknown,
            availability: SectionAvailability::Available,
            coverage: SectionCoverage::Partial,
            consistency: SourceConsistency::ObservedCountFillOnly,
            stable_identity: true,
        })
        .expect("zero-member observation finishes");
    producer.progress(1, 0).expect("valid test fixture");
    take_messages(producer, &source, revision, &key)
}
