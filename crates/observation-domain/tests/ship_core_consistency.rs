use observation_domain::{
    SenderEvidence, ShipClass, ShipCoreRecord, ShipIdentity, ShipLocation, ShipOwner,
    ShipRecordConsistency, ShipStaleReason, ShipType, SourceScopeId,
};

#[test]
fn core_defaults_consistent_and_preserves_an_explicit_stale_reason() {
    let record = ShipCoreRecord::new(
        SourceScopeId::new("x4:faction:argon:ships").expect("scope"),
        ShipIdentity::new("9007199254740993").expect("identity"),
        ShipOwner::new("argon").expect("owner"),
        ShipType::new("destroyer_macro").expect("type"),
        ShipClass::new("destroyer").expect("class"),
        ShipLocation::new("sector:argon_prime").expect("location"),
        SenderEvidence::legacy_default(),
    );
    assert_eq!(record.consistency(), ShipRecordConsistency::Consistent);
    let stale = ShipRecordConsistency::PossiblyStale(ShipStaleReason::OwnerChanged);
    assert_eq!(record.with_consistency(stale).consistency(), stale);
}
