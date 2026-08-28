use observation_ingest::{AcceptedProjection, admit_batch};
use strategic_state::{Faction, PacketLimits, ThreatSubject, derive_packets};

const FRAMES: [&str; 17] = [
    r#"{"type":"observation","scope":"ZYA","entity_id":"ZYA:economy:ore","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#,
    r#"{"type":"observation","scope":"ARG","entity_id":"ARG:economy:ore","observed_at_unix_millis":1,"version":1,"quality":"stale","content":"own"}"#,
    r#"{"type":"observation","scope":"XEN","entity_id":"XEN:economy:ore","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"foreign"}"#,
    r#"{"type":"observation","scope":"KHK","entity_id":"KHK:economy:ore","observed_at_unix_millis":1,"version":1,"quality":"unsupported","content":"foreign"}"#,
    r#"{"type":"observation","scope":"ZYA","entity_id":"ZYA:military:fleet","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#,
    r#"{"type":"observation","scope":"ARG","entity_id":"ARG:military:fleet","observed_at_unix_millis":1,"version":1,"quality":"stale","content":"own"}"#,
    r#"{"type":"observation","scope":"XEN","entity_id":"XEN:military:fleet","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"foreign"}"#,
    r#"{"type":"observation","scope":"KHK","entity_id":"KHK:military:fleet","observed_at_unix_millis":1,"version":1,"quality":"unsupported","content":"foreign"}"#,
    r#"{"type":"observation","scope":"ZYA","entity_id":"ZYA:territorial:station","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#,
    r#"{"type":"observation","scope":"ARG","entity_id":"ARG:territorial:station","observed_at_unix_millis":1,"version":1,"quality":"stale","content":"own"}"#,
    r#"{"type":"observation","scope":"XEN","entity_id":"XEN:territorial:station","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"foreign"}"#,
    r#"{"type":"observation","scope":"KHK","entity_id":"KHK:territorial:station","observed_at_unix_millis":1,"version":1,"quality":"unsupported","content":"foreign"}"#,
    r#"{"type":"observation","scope":"ZYA","entity_id":"ZYA:threat:XEN","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"shared"}"#,
    r#"{"type":"observation","scope":"ARG","entity_id":"ARG:threat:XEN","observed_at_unix_millis":1,"version":1,"quality":"stale","content":"shared"}"#,
    r#"{"type":"observation","scope":"KHK","entity_id":"KHK:threat:KHK","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"observed"}"#,
    r#"{"type":"observation","scope":"ZYA","entity_id":"ZYA:threat:KHK","observed_at_unix_millis":1,"version":1,"quality":"unsupported","content":"unsupported"}"#,
    r#"{"type":"observation","scope":"ARG","entity_id":"ARG:threat:KHK","observed_at_unix_millis":1,"version":1,"quality":"unsupported","content":"unsupported"}"#,
];

fn projection() -> observation_ingest::ProjectionSnapshot {
    admit_batch(AcceptedProjection::empty(), &FRAMES)
        .into_projection()
        .snapshot
}

#[test]
fn derives_paired_packets_with_explicit_four_family_availability() {
    let packets = derive_packets(&projection(), PacketLimits::tracer()).expect("fixture is valid");
    assert_eq!(packets.policy_version(), "visibility-v1");
    assert_eq!(packets.packet(Faction::Zya).facts().len(), 17);
    assert_eq!(packets.packet(Faction::Arg).facts().len(), 17);
    assert!(packets.packet(Faction::Zya).has_shared_threat(ThreatSubject::Xen));
    assert!(packets.packet(Faction::Arg).has_observed_threat(ThreatSubject::Khk));
}

#[test]
fn rejects_capacity_before_packet_creation() {
    assert!(derive_packets(&projection(), PacketLimits::new(1, 1)).is_err());
}
