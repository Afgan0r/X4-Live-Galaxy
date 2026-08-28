use observation_domain::{
    EntityId, ObservationSource, ObservationTime, ObservationVersion, SectionDescriptor,
    SectionQuality,
};

#[test]
fn foundation_contract_preserves_typed_identity_and_section_quality() {
    let entity_id = EntityId::new("sector:alpha").expect("a fixed entity id is valid");
    let source = ObservationSource::x4_runtime();
    let observed_at = ObservationTime::from_unix_millis(1_725_000_000_000);
    let version = ObservationVersion::new(7).expect("a positive version is valid");

    let declared_qualities = [
        SectionQuality::KnownEmpty,
        SectionQuality::Unknown,
        SectionQuality::Partial,
        SectionQuality::Stale,
        SectionQuality::Unsupported,
    ];

    for quality in declared_qualities {
        let descriptor =
            SectionDescriptor::new(entity_id.clone(), source, observed_at, version, quality);

        assert_eq!(descriptor.entity_id(), &entity_id);
        assert_eq!(descriptor.source(), &source);
        assert_eq!(descriptor.observed_at(), observed_at);
        assert_eq!(descriptor.version(), version);
        assert_eq!(descriptor.quality(), quality);
    }
}
