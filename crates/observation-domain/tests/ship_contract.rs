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

fn cargo_content() -> String {
    "profile=ship_cargo\nidentity=9007199254740993\nowner=argon\ncore_revision=7\nmember_revision=7\npolicy=2\ncapture_start=100\ncapture_end=110\nsource=x4-9.00-steam-23660954-ship-detail-source-v1\nconsistency=consistent\nconsistency_reason=none\nwares_outcome=value\nstorage_outcome=value\nreservation_policy=excluded\nware=ore|17\nstorage=solid|1000|170".to_owned()
}

#[test]
fn cargo_roundtrip_keeps_items_and_volume_independent() {
    let text = cargo_content();
    let cargo =
        observation_domain::CargoObservation::from_content(&text, 2).expect("cargo is valid");
    assert_eq!(cargo.canonical_content(), text);
    assert_eq!(cargo.dependency.identity.as_str(), "9007199254740993");
    let FieldOutcome::Value(wares) = cargo.wares else {
        panic!("wares are observed");
    };
    let FieldOutcome::Value(storage) = cargo.storage else {
        panic!("storage is observed");
    };
    assert_eq!(wares[0].amount_items, 17);
    assert_eq!(storage[0].occupied_cubic_metres, 170);
    assert_eq!(storage[0].capacity_cubic_metres, 1000);
}

#[test]
fn cargo_rejects_wrong_dependencies_unsafe_numbers_and_inner_overflow() {
    let text = cargo_content();
    for (from, to) in [
        ("member_revision=7", "member_revision=8"),
        ("policy=2", "policy=3"),
        ("capture_end=110", "capture_end=99"),
        ("ore|17", "ore|9007199254740992"),
        ("ore|17", "ore|-1"),
        ("ore|17", "ore|01"),
        ("solid|1000|170", "solid|1000|1001"),
        ("solid|1000|170", "solid|4294967296|170"),
        ("wares_outcome=value", "wares_outcome=unknown"),
    ] {
        assert!(
            observation_domain::CargoObservation::from_content(&text.replace(from, to), 2).is_err(),
            "must reject {to}"
        );
    }
    assert!(observation_domain::CargoObservation::from_content(&text, 0).is_err());
    assert!(
        observation_domain::CargoObservation::from_content(
            &format!("{text}\nstorage=solid|1000|1"),
            2
        )
        .is_err()
    );
}

#[test]
fn cargo_preserves_every_nonvalue_outcome_without_fabricated_rows() {
    let text = cargo_content();
    let prefix = text.split("\nware=").next().expect("header exists");
    for outcome in [
        "zero",
        "empty",
        "absent",
        "unknown",
        "inaccessible",
        "unsupported",
        "stale",
    ] {
        let text = prefix
            .replace("wares_outcome=value", &format!("wares_outcome={outcome}"))
            .replace(
                "storage_outcome=value",
                &format!("storage_outcome={outcome}"),
            );
        let cargo = observation_domain::CargoObservation::from_content(&text, 2)
            .expect("explicit outcome is valid");
        assert_eq!(cargo.canonical_content(), text);
    }
}

#[test]
fn detail_groups_freeze_the_exact_parent_members_and_bounded_partition() {
    let parent = ShipGroupDescriptor::new(
        scope(),
        SectionRevisionId::new(7).expect("revision"),
        ObservationPolicyVersion::new(2).expect("policy"),
        [
            identity("9007199254740995"),
            identity("9007199254740993"),
            identity("9007199254740994"),
        ],
    )
    .expect("parent");
    let first = observation_domain::ShipDetailGroup::new(parent.clone(), 0, 2).expect("group");
    let second = observation_domain::ShipDetailGroup::new(parent.clone(), 1, 2).expect("group");
    assert_eq!(first.parent(), &parent);
    assert_eq!(
        first.members(),
        &[identity("9007199254740993"), identity("9007199254740994")]
    );
    assert_eq!(second.members(), &[identity("9007199254740995")]);
    assert!(observation_domain::ShipDetailGroup::new(parent.clone(), 2, 2).is_err());
    assert!(observation_domain::ShipDetailGroup::new(parent, 0, 0).is_err());
}
