#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "contract fixtures fail immediately"
)]
use observation_domain::*;
use x4_carrier_native::{SectionEvidence, SectionFinishEvidence};
mod transport;
pub use transport::{ready, take_messages};

pub fn messages(revision: u64, owner: &str) -> Vec<Vec<u8>> {
    with_identities(revision, owner, ["9007199254740993", "9007199254740995"])
}

pub fn with_identities(revision: u64, owner: &str, identities: [&str; 2]) -> Vec<Vec<u8>> {
    with_core_consistency(revision, owner, identities, None)
}

pub fn with_core_consistency(
    revision: u64,
    owner: &str,
    identities: [&str; 2],
    first_consistency: Option<ShipRecordConsistency>,
) -> Vec<Vec<u8>> {
    core_messages(
        revision,
        "argon",
        owner,
        identities,
        first_consistency,
        "ship_core",
    )
}

pub fn core_messages(
    revision: u64,
    source_faction: &str,
    owner: &str,
    identities: [&str; 2],
    first_consistency: Option<ShipRecordConsistency>,
    key: &str,
) -> Vec<Vec<u8>> {
    let (mut producer, source) = ready(revision, source_faction, key);
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
        .begin_ship_section(evidence.clone(), 2)
        .expect("valid test fixture");
    for (index, id) in identities.into_iter().enumerate() {
        let mut record = ShipCoreRecord::new(
            SourceScopeId::new(source.source_scope.clone()).expect("valid test fixture"),
            ShipIdentity::new(id).expect("valid test fixture"),
            ShipOwner::new(owner).expect("valid test fixture"),
            ShipType::new("destroyer_macro").expect("valid test fixture"),
            ShipClass::new("destroyer").expect("valid test fixture"),
            ShipLocation::new("sector:2").expect("valid test fixture"),
            evidence.sender.clone(),
        );
        if index == 0
            && let Some(consistency) = first_consistency
        {
            record = record.with_consistency(consistency);
        }
        producer
            .push_ship_core(&record)
            .expect("valid test fixture");
    }
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
        .expect("valid test fixture");
    producer.progress(1, 0).expect("valid test fixture");
    take_messages(producer, &source, revision, key)
}
