use strategic_state::{Faction, PacketLimits, ShadowPrimitive, derive_packets};

mod support;

const ORDERED: [&str; 4] = [
    r#"{"type":"observation","scope":"ZYA","entity_id":"ZYA:military:fleet","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#,
    r#"{"type":"observation","scope":"ZYA","entity_id":"ZYA:economy:ore","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#,
    r#"{"type":"observation","scope":"ZYA","entity_id":"ZYA:territorial:station","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#,
    r#"{"type":"observation","scope":"XEN","entity_id":"XEN:threat:XEN","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"shared"}"#,
];

fn packet(
    frames: &[&str],
) -> Result<strategic_state::StrategicPacket, strategic_state::DerivationError> {
    let snapshot = support::admit_runtime_fact_frames(frames).snapshot;
    derive_packets(&snapshot, PacketLimits::tracer())
        .map(|packets| packets.packet(Faction::Zya).clone())
}

#[test]
fn permutations_have_one_canonical_packet_and_replay_identity() {
    let ordered_result = packet(&ORDERED);
    let permuted_result = packet(&[ORDERED[3], ORDERED[1], ORDERED[0], ORDERED[2]]);
    assert!(ordered_result.is_ok());
    assert!(permuted_result.is_ok());
    let Ok(ordered) = ordered_result else { return };
    let Ok(permuted) = permuted_result else {
        return;
    };
    let ordered_result = ShadowPrimitive::derive(&ordered);
    let permuted_result = ShadowPrimitive::derive(&permuted);
    assert!(ordered_result.is_ok());
    assert!(permuted_result.is_ok());
    let Ok(ordered_primitives) = ordered_result else {
        return;
    };
    let Ok(permuted_primitives) = permuted_result else {
        return;
    };

    assert_eq!(ordered.canonical_facts(), permuted.canonical_facts());
    assert_eq!(
        ordered.admission_inputs(&ordered_primitives),
        permuted.admission_inputs(&permuted_primitives)
    );
    assert_eq!(
        ordered.replay_fingerprint(&ordered_primitives),
        permuted.replay_fingerprint(&permuted_primitives)
    );
}

#[test]
fn configured_primitive_budget_rejects_below_four() {
    let snapshot = support::admit_runtime_fact_frames(&ORDERED).snapshot;
    for limit in [1, 2, 3] {
        assert_eq!(
            derive_packets(&snapshot, PacketLimits::new(32, limit)),
            Err(strategic_state::DerivationError::PrimitiveLimitExceeded)
        );
    }
    assert!(derive_packets(&snapshot, PacketLimits::new(32, 4)).is_ok());
    assert!(derive_packets(&snapshot, PacketLimits::new(32, 5)).is_ok());
}

#[test]
fn replay_fingerprint_changes_for_retained_identity_and_version() {
    let baseline = packet(&ORDERED).expect("baseline packet");
    let primitives = ShadowPrimitive::derive(&baseline).expect("baseline primitives");
    let fingerprint = baseline.replay_fingerprint(&primitives);
    for changed in [
        r#"{"type":"observation","scope":"ZYA","entity_id":"ZYA:economy:energy","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#,
        r#"{"type":"observation","scope":"ZYA","entity_id":"ZYA:economy:ore","observed_at_unix_millis":1,"version":2,"quality":"fresh","content":"own"}"#,
    ] {
        let candidate =
            packet(&[ORDERED[0], changed, ORDERED[2], ORDERED[3]]).expect("candidate packet");
        let primitives = ShadowPrimitive::derive(&candidate).expect("candidate primitives");
        assert_ne!(fingerprint, candidate.replay_fingerprint(&primitives));
    }
}
