use observation_ingest::{AcceptedProjection, admit_batch};
use strategic_state::{Capability, Faction, PacketLimits, derive_packets};

const FRAMES: [&str; 3] = [
    r#"{"type":"observation","scope":"ZYA","entity_id":"ZYA:military:fleet","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#,
    r#"{"type":"observation","scope":"ARG","entity_id":"ARG:economy:ore","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#,
    r#"{"type":"observation","scope":"XEN","entity_id":"XEN:threat:XEN","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"shared"}"#,
];

fn projection() -> observation_ingest::ProjectionSnapshot {
    admit_batch(AcceptedProjection::empty(), &FRAMES)
        .into_projection()
        .snapshot
}

#[test]
fn versioned_profiles_keep_locked_live_galaxy_labels_and_priority_policy() {
    let packets = derive_packets(&projection(), PacketLimits::tracer()).expect("fixture is valid");
    let zya = packets.packet(Faction::Zya);
    let arg = packets.packet(Faction::Arg);

    assert_eq!(zya.profile_version(), "doctrine-v1");
    assert_eq!(arg.profile_version(), "doctrine-v1");
    assert!(zya.profile().is_live_galaxy_product_policy());
    assert!(arg.profile().is_live_galaxy_product_policy());
    assert!(!zya.profile().labels_are_official_x4_names());
    assert_eq!(
        zya.profile().priorities(),
        [
            Capability::DefenseAndMilitaryStrategy,
            Capability::TerritorialDevelopmentAndInfrastructure,
            Capability::EconomyAndLogistics,
        ]
    );
    assert_eq!(
        arg.profile().priorities(),
        [
            Capability::EconomyAndLogistics,
            Capability::DefenseAndMilitaryStrategy,
            Capability::TerritorialDevelopmentAndInfrastructure,
        ]
    );
    assert_eq!(
        zya.institution_views()
            .iter()
            .map(strategic_state::InstitutionView::label)
            .collect::<Vec<_>>(),
        [
            "ZYA Defense & Military Strategy",
            "ZYA Economy & Logistics",
            "ZYA Territorial Development & Infrastructure",
        ]
    );
    assert_eq!(
        arg.institution_views()
            .iter()
            .map(strategic_state::InstitutionView::label)
            .collect::<Vec<_>>(),
        [
            "ARG Defense & Military Strategy",
            "ARG Economy & Logistics",
            "ARG Territorial Development & Infrastructure",
        ]
    );
}
