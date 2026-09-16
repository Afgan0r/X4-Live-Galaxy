#![expect(
    clippy::expect_used,
    reason = "test fixtures fail immediately when their invariants are invalid"
)]

use observation_domain::{
    FieldApplicability, FieldOutcome, ObservationPolicyVersion, SectionRevisionId, SenderEvidence,
    ShipClass, ShipCoreError, ShipCoreRecord, ShipGroupDescriptor, ShipGroupError, ShipIdentity,
    ShipLocation, ShipOwner, ShipType, SourceEvidenceRef, SourceScopeId,
};

fn identity(value: &str) -> ShipIdentity {
    ShipIdentity::new(value).expect("fixture identity is canonical")
}

fn scope() -> SourceScopeId {
    SourceScopeId::new("x4:faction:argon:ships").expect("fixture scope is valid")
}

#[test]
fn core_requires_identity_owner_type_class_and_location() {
    assert_eq!(ShipIdentity::new(""), Err(ShipCoreError::MissingIdentity));
    assert_eq!(
        ShipIdentity::new(" 42"),
        Err(ShipCoreError::InvalidIdentity)
    );
    assert_eq!(
        ShipIdentity::new("0042"),
        Err(ShipCoreError::InvalidIdentity)
    );
    assert_eq!(ShipOwner::new(""), Err(ShipCoreError::MissingOwner));
    assert_eq!(ShipType::new(" "), Err(ShipCoreError::MissingType));
    assert_eq!(ShipClass::new(""), Err(ShipCoreError::MissingClass));
    assert_eq!(ShipLocation::new("\t"), Err(ShipCoreError::MissingLocation));

    let record = ShipCoreRecord::new(
        scope(),
        identity("9007199254740993"),
        ShipOwner::new("argon").expect("owner is valid"),
        ShipType::new("ship_arg_l_destroyer_01_a_macro").expect("type is valid"),
        ShipClass::new("destroyer").expect("class is valid"),
        ShipLocation::new("sector:argon_prime").expect("location is valid"),
        SenderEvidence::legacy_default(),
    );

    assert_eq!(record.identity().as_str(), "9007199254740993");
    assert_eq!(record.owner().as_str(), "argon");
    assert_eq!(record.location().as_str(), "sector:argon_prime");
}

#[test]
fn field_states_preserve_each_locked_distinction() {
    let evidence = SourceEvidenceRef::new("x4:GetComponentData:macro").expect("evidence exists");
    assert_eq!(
        FieldApplicability::NotApplicable(evidence.clone()),
        FieldApplicability::NotApplicable(evidence)
    );
    assert_ne!(FieldOutcome::<u64>::Zero, FieldOutcome::Empty);
    assert_ne!(FieldOutcome::<u64>::Empty, FieldOutcome::Absent);
    assert_ne!(FieldOutcome::<u64>::Absent, FieldOutcome::Unknown);
    assert_ne!(FieldOutcome::<u64>::Unknown, FieldOutcome::Inaccessible);
    assert_ne!(FieldOutcome::<u64>::Inaccessible, FieldOutcome::Unsupported);
    assert_ne!(FieldOutcome::<u64>::Unsupported, FieldOutcome::Stale);
    assert!(SourceEvidenceRef::new("").is_none());
}

#[test]
fn group_order_is_canonical_and_duplicate_identity_is_rejected() {
    let revision = SectionRevisionId::new(7).expect("revision is positive");
    let policy = ObservationPolicyVersion::new(3).expect("policy is positive");
    let first = ShipGroupDescriptor::new(
        scope(),
        revision,
        policy,
        [identity("20"), identity("3"), identity("100")],
    )
    .expect("unique group is valid");
    let second = ShipGroupDescriptor::new(
        scope(),
        revision,
        policy,
        [identity("100"), identity("20"), identity("3")],
    )
    .expect("unique group is valid");

    assert_eq!(first.members(), second.members());
    assert_eq!(
        first.members(),
        &[identity("100"), identity("20"), identity("3")]
    );
    assert_eq!(
        ShipGroupDescriptor::new(scope(), revision, policy, [identity("20"), identity("20")]),
        Err(ShipGroupError::DuplicateIdentity(identity("20")))
    );
}
