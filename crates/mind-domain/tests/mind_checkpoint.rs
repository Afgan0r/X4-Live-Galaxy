use mind_domain::{
    Capability, CommandId, InitiativeCommand, InitiativeId, InitiativeSpec, MindAggregate,
    MindCheckpointState, PreemptionDisposition, transition,
};
use observation_ingest::{AcceptedProjection, admit_batch};
use serde_json::Value;
use strategic_state::{Faction, PacketLimits, derive_packets};

const FRAMES: [&str; 4] = [
    r#"{"type":"observation","scope":"ZYA","entity_id":"ZYA:military:fleet","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#,
    r#"{"type":"observation","scope":"ARG","entity_id":"ARG:economy:ore","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#,
    r#"{"type":"observation","scope":"ZYA","entity_id":"ZYA:territorial:station","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#,
    r#"{"type":"observation","scope":"XEN","entity_id":"XEN:threat:XEN","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"shared"}"#,
];

fn transitioned() -> MindAggregate {
    let snapshot = admit_batch(AcceptedProjection::empty(), &FRAMES)
        .into_projection()
        .snapshot;
    let packets = derive_packets(&snapshot, PacketLimits::tracer());
    assert!(packets.is_ok());
    let Ok(packets) = packets else {
        return MindAggregate::empty(Faction::Zya);
    };
    let mind = transition(
        &MindAggregate::empty(Faction::Zya),
        mind_domain::MindCommand::from_packet(
            packets.packet(Faction::Zya),
            CommandId::new("mind-checkpoint"),
        ),
    );
    assert!(mind.is_ok());
    let Ok(mind) = mind else {
        return MindAggregate::empty(Faction::Zya);
    };
    let first = accepted(mind.aggregate());
    let replacement = preempted(first.aggregate());
    let terminal = completed(replacement.aggregate());
    terminal.aggregate().clone()
}

fn accepted(mind: &MindAggregate) -> mind_domain::PendingInitiativeCommit {
    let first = mind.apply_initiative(InitiativeCommand::accept(
        CommandId::new("accept-a"),
        InitiativeSpec::new(
            InitiativeId::new("initiative-a"),
            Capability::DefenseAndMilitaryStrategy,
            "defend frontier",
            "military-fact",
            90,
        ),
    ));
    assert!(first.is_ok());
    let Ok(first) = first else { unreachable!() };
    first
}

fn preempted(mind: &MindAggregate) -> mind_domain::PendingInitiativeCommit {
    let replacement = mind.apply_initiative(InitiativeCommand::preempt(
        CommandId::new("preempt-b"),
        InitiativeId::new("initiative-a"),
        InitiativeSpec::new(
            InitiativeId::new("initiative-b"),
            Capability::DefenseAndMilitaryStrategy,
            "defend frontier",
            "military-fact",
            95,
        ),
        "new threat evidence",
        PreemptionDisposition::Cancelled,
    ));
    assert!(replacement.is_ok());
    let Ok(replacement) = replacement else {
        unreachable!()
    };
    replacement
}

fn completed(mind: &MindAggregate) -> mind_domain::PendingInitiativeCommit {
    let terminal = mind.apply_initiative(InitiativeCommand::complete(
        CommandId::new("terminal-b"),
        InitiativeId::new("initiative-b"),
    ));
    assert!(terminal.is_ok());
    let Ok(terminal) = terminal else {
        unreachable!()
    };
    terminal
}

fn state() -> MindCheckpointState {
    let aggregate = transitioned();
    let snapshot = admit_batch(AcceptedProjection::empty(), &FRAMES)
        .into_projection()
        .snapshot;
    let result = derive_packets(&snapshot, PacketLimits::tracer());
    assert!(result.is_ok());
    let Ok(packets) = result else { unreachable!() };
    let pending = transition(
        &aggregate,
        mind_domain::MindCommand::from_packet(
            packets.packet(Faction::Zya),
            CommandId::new("mind-checkpoint"),
        ),
    );
    assert!(pending.is_ok());
    let Ok(pending) = pending else { unreachable!() };
    pending.checkpoint_state()
}

#[test]
fn canonical_checkpoint_round_trips_replayed_full_mind() {
    let state = state();
    let json = serde_json::to_string(&state);
    assert!(json.is_ok());
    let Ok(json) = json else { return };
    let decoded: Result<MindCheckpointState, _> = serde_json::from_str(&json);
    assert!(decoded.is_ok());
    let Ok(decoded) = decoded else { return };
    let restored = decoded.restore();
    assert!(restored.is_ok());
    let Ok(restored) = restored else { return };
    assert_eq!(restored, transitioned());
    let replayed_json = serde_json::to_string(&decoded);
    assert!(replayed_json.is_ok());
    assert_eq!(replayed_json.ok(), Some(json));
}

#[test]
fn malformed_oversized_and_unknown_checkpoint_is_rejected() {
    let checkpoint = state();
    let value = serde_json::to_value(&checkpoint);
    assert!(value.is_ok());
    let Ok(mut value) = value else { return };
    if let Value::Object(object) = &mut value {
        object.insert("unknown".into(), Value::Null);
    }
    assert!(serde_json::from_value::<MindCheckpointState>(value).is_err());

    let checkpoint = state();
    let value = serde_json::to_value(&checkpoint);
    assert!(value.is_ok());
    let Ok(mut value) = value else { return };
    value["commit"]["aggregate"]["motives"][0] = Value::String("x".repeat(257));
    let decoded: Result<MindCheckpointState, _> = serde_json::from_value(value);
    assert!(decoded.is_ok());
    let Ok(decoded) = decoded else { return };
    assert!(decoded.restore().is_err());

    let checkpoint = state();
    let value = serde_json::to_value(&checkpoint);
    assert!(value.is_ok());
    let Ok(mut value) = value else { return };
    value["commit"]["aggregate"]["goals"][0] = Value::String("EconomyAndLogistics".into());
    let decoded: Result<MindCheckpointState, _> = serde_json::from_value(value);
    assert!(decoded.is_ok());
    let Ok(decoded) = decoded else { return };
    assert!(decoded.restore().is_err());
}
