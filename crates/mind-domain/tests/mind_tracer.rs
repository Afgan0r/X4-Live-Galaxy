use mind_domain::{CommandId, MindAggregate, MindCommand, transition};
use observation_ingest::{AcceptedProjection, admit_batch};
use strategic_state::{Faction, PacketLimits, derive_packets};

const FRAMES: [&str; 4] = [
    r#"{"type":"observation","scope":"ZYA","entity_id":"ZYA:military:fleet","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#,
    r#"{"type":"observation","scope":"ARG","entity_id":"ARG:economy:ore","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#,
    r#"{"type":"observation","scope":"ZYA","entity_id":"ZYA:territorial:station","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#,
    r#"{"type":"observation","scope":"XEN","entity_id":"XEN:threat:XEN","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"shared"}"#,
];

fn packets() -> strategic_state::PairedPackets {
    let snapshot = admit_batch(AcceptedProjection::empty(), &FRAMES)
        .into_projection()
        .snapshot;
    let result = derive_packets(&snapshot, PacketLimits::tracer());
    assert!(result.is_ok());
    let Ok(packets) = result else { unreachable!() };
    packets
}

#[test]
fn creates_independent_doctrine_divergent_replayable_minds() {
    let packets = packets();
    let zya = packets.packet(Faction::Zya);
    let arg = packets.packet(Faction::Arg);
    let zya_command = MindCommand::from_packet(zya, CommandId::new("mind-zya-1"));
    let arg_command = MindCommand::from_packet(arg, CommandId::new("mind-arg-1"));

    let zya_commit = transition(&MindAggregate::empty(Faction::Zya), zya_command);
    let arg_commit = transition(&MindAggregate::empty(Faction::Arg), arg_command);

    assert!(zya_commit.is_ok());
    assert!(arg_commit.is_ok());
    let Ok(zya_commit) = zya_commit else { return };
    let Ok(arg_commit) = arg_commit else { return };
    assert_eq!(zya_commit.aggregate().doctrine_version(), "doctrine-v1");
    assert_eq!(zya_commit.aggregate().motives().len(), 2);
    assert_eq!(zya_commit.aggregate().priorities().len(), 3);
    assert_eq!(zya_commit.aggregate().goals().len(), 3);
    assert_eq!(zya_commit.aggregate().short_term_plans().len(), 3);
    assert_eq!(zya_commit.aggregate().long_term_plans().len(), 3);
    assert_ne!(
        zya_commit.aggregate().priorities(),
        arg_commit.aggregate().priorities()
    );
    assert_ne!(
        zya_commit.aggregate().posture(),
        arg_commit.aggregate().posture()
    );

    let replay = transition(
        &MindAggregate::empty(Faction::Zya),
        MindCommand::from_packet(zya, CommandId::new("mind-zya-1")),
    );
    assert_eq!(Ok(zya_commit), replay);
}
