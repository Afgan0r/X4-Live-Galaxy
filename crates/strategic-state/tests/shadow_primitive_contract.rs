use strategic_state::{
    BilateralPosture, Faction, PacketLimits, PlanningHorizon, PrimitiveOwner, ShadowPrimitive,
    ShadowPrimitiveError, ShadowPrimitiveKind, derive_packets,
};

mod support;

const FRAMES: [&str; 4] = [
    r#"{"type":"observation","scope":"ZYA","entity_id":"ZYA:military:fleet","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#,
    r#"{"type":"observation","scope":"ZYA","entity_id":"ZYA:economy:ore","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#,
    r#"{"type":"observation","scope":"ZYA","entity_id":"ZYA:territorial:station","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#,
    r#"{"type":"observation","scope":"XEN","entity_id":"XEN:threat:XEN","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"shared"}"#,
];

#[test]
fn finite_allowlist_is_bounded_and_executive_posture_is_not_an_institution_view() {
    let snapshot = support::admit_runtime_fact_frames(&FRAMES).snapshot;
    let packets = derive_packets(&snapshot, PacketLimits::tracer()).expect("accepted fixture");
    let packet = packets.packet(Faction::Zya);
    let primitives = ShadowPrimitive::derive(packet).expect("available visible facts");

    assert_eq!(primitives.len(), 4);
    assert_eq!(
        primitives
            .iter()
            .map(ShadowPrimitive::kind)
            .collect::<Vec<_>>(),
        vec![
            ShadowPrimitiveKind::DefensiveReadiness,
            ShadowPrimitiveKind::LogisticsAllocationPriority,
            ShadowPrimitiveKind::TerritorialDevelopmentPriority,
            ShadowPrimitiveKind::BilateralPostureDisposition,
        ]
    );
    assert!(
        primitives
            .iter()
            .all(|primitive| primitive.priority() >= 1 && primitive.priority() <= 100)
    );
    assert!(
        primitives
            .iter()
            .all(|primitive| primitive.evidence().len() <= 8)
    );
    assert_eq!(primitives[3].owner(), PrimitiveOwner::Executive);
    assert_eq!(
        primitives[3].posture(),
        Some(BilateralPosture::PreserveRelations)
    );
    assert_eq!(primitives[3].horizon(), PlanningHorizon::NearTerm);
    assert!(packet.has_shared_threat(strategic_state::ThreatSubject::Xen));
    assert_eq!(
        ShadowPrimitive::reject_unknown_kind("execute_effect"),
        Err(ShadowPrimitiveError::UnsupportedKind)
    );
}

#[test]
fn rejects_required_primitive_evidence_when_quality_is_unavailable() {
    for quality in ["unknown", "stale", "unsupported"] {
        let military = format!(
            r#"{{"type":"observation","scope":"ZYA","entity_id":"ZYA:military:fleet","observed_at_unix_millis":1,"version":1,"quality":"{quality}","content":"unavailable"}}"#
        );
        let frames = [
            military,
            r#"{"type":"observation","scope":"ZYA","entity_id":"ZYA:economy:ore","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#.to_owned(),
            r#"{"type":"observation","scope":"ZYA","entity_id":"ZYA:territorial:station","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#.to_owned(),
            r#"{"type":"observation","scope":"XEN","entity_id":"XEN:threat:XEN","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"shared"}"#.to_owned(),
        ];
        let input = frames.iter().map(String::as_str).collect::<Vec<_>>();
        let snapshot = support::admit_runtime_fact_frames(&input).snapshot;
        let packets = derive_packets(&snapshot, PacketLimits::tracer()).expect("packet input");

        assert_eq!(
            ShadowPrimitive::derive(packets.packet(Faction::Zya)),
            Err(ShadowPrimitiveError::UnavailableRequiredFact),
            "military quality {quality} must not become a usable primitive input"
        );
    }
}
