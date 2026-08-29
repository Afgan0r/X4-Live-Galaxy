use strategic_state::{
    FactAvailability, FactFamily, Faction, PacketLimits, VisibilityPolicy, derive_with_policy,
};

mod support;

const FRAMES: [&str; 4] = [
    r#"{"type":"observation","scope":"ZYA","entity_id":"ZYA:economy:ore","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#,
    r#"{"type":"observation","scope":"ARG","entity_id":"ARG:military:fleet","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"foreign-changing"}"#,
    r#"{"type":"observation","scope":"ARG","entity_id":"ARG:territorial:resource_map","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"foreign-static"}"#,
    r#"{"type":"observation","scope":"XEN","entity_id":"XEN:threat:XEN","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"shared"}"#,
];

fn projection() -> observation_ingest::ProjectionSnapshot {
    support::admit_runtime_fact_frames(&FRAMES).snapshot
}

#[test]
fn policies_filter_before_derivation_and_preserve_phase_one_quality_assumption() {
    let packets = derive_with_policy(
        &projection(),
        VisibilityPolicy::v1(),
        PacketLimits::tracer(),
    )
    .expect("Phase 1 quality mapping is a local runtime-semantics assumption");
    assert_eq!(
        packets.packet(Faction::Zya).policy_version(),
        "visibility-v1"
    );
    assert_eq!(
        packets.packet(Faction::Arg).policy_version(),
        "visibility-v1"
    );
    assert_eq!(
        packets
            .packet(Faction::Zya)
            .availability(FactFamily::Military),
        FactAvailability::Inaccessible
    );
    assert_eq!(
        packets
            .packet(Faction::Zya)
            .availability(FactFamily::Territorial),
        FactAvailability::Available
    );
    assert_eq!(
        packets
            .packet(Faction::Arg)
            .availability(FactFamily::Economic),
        FactAvailability::Inaccessible
    );
}
