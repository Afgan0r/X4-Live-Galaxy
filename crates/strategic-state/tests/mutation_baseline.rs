use strategic_state::{
    FactAvailability, FactFamily, FactOwner, Faction, PacketLimits, StrategicFact, derive_packets,
};

mod support;

const FRAMES: [&str; 4] = [
    r#"{"type":"observation","scope":"ZYA","entity_id":"ZYA:military:fleet","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#,
    r#"{"type":"observation","scope":"ARG","entity_id":"ARG:economy:ore","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"foreign"}"#,
    r#"{"type":"observation","scope":"XEN","entity_id":"XEN:threat:XEN","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"shared"}"#,
    r#"{"type":"observation","scope":"KHK","entity_id":"KHK:threat:KHK","observed_at_unix_millis":1,"version":1,"quality":"unsupported","content":"unavailable"}"#,
];

#[test]
fn visibility_availability_capacity_and_canonical_order_remain_observable() {
    let snapshot = support::admit_runtime_fact_frames(&FRAMES).snapshot;
    let result = derive_packets(&snapshot, PacketLimits::tracer());
    assert!(result.is_ok());
    let Ok(packets) = result else { return };
    let zya = packets.packet(Faction::Zya);
    let arg = packets.packet(Faction::Arg);

    let zya_foreign_economy = zya.facts().iter().find(|fact| {
        fact.reference().owner() == FactOwner::Arg
            && fact.reference().family() == FactFamily::Economic
    });
    let arg_foreign_economy = arg.facts().iter().find(|fact| {
        fact.reference().owner() == FactOwner::Arg
            && fact.reference().family() == FactFamily::Economic
    });
    assert!(zya_foreign_economy.is_some());
    assert!(arg_foreign_economy.is_some());
    let Some(zya_foreign_economy) = zya_foreign_economy else {
        return;
    };
    let Some(arg_foreign_economy) = arg_foreign_economy else {
        return;
    };
    assert_eq!(
        zya_foreign_economy.availability(),
        FactAvailability::Inaccessible
    );
    assert_eq!(
        arg_foreign_economy.availability(),
        FactAvailability::Available
    );
    assert_eq!(
        zya.canonical_facts()
            .iter()
            .map(StrategicFact::reference)
            .collect::<Vec<_>>(),
        arg.canonical_facts()
            .iter()
            .map(StrategicFact::reference)
            .collect::<Vec<_>>(),
    );
    assert!(derive_packets(&snapshot, PacketLimits::new(3, 4)).is_err());
}
