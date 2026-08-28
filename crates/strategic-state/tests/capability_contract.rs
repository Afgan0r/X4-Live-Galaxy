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
fn exposes_only_the_three_shared_capabilities_from_one_visible_snapshot() {
    let packets = derive_packets(&projection(), PacketLimits::tracer()).expect("fixture is valid");
    let capabilities = [
        Capability::DefenseAndMilitaryStrategy,
        Capability::EconomyAndLogistics,
        Capability::TerritorialDevelopmentAndInfrastructure,
    ];

    assert_eq!(Capability::ALL, capabilities);
    for faction in [Faction::Zya, Faction::Arg] {
        let packet = packets.packet(faction);
        let views = packet.institution_views();
        assert_eq!(views.len(), capabilities.len());
        assert_eq!(
            views
                .iter()
                .map(strategic_state::InstitutionView::capability)
                .collect::<Vec<_>>(),
            capabilities
        );
        assert!(
            views
                .iter()
                .all(|view| view.snapshot_id() == packet.visible_snapshot_id())
        );
    }
}
