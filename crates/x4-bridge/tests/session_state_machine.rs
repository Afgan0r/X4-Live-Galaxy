use x4_bridge::{
    CapabilityDecision, RestartRequirement, SequenceNumber, SessionGeneration, SessionHello,
    SessionState,
};

fn compatible_hello() -> SessionHello {
    SessionHello::new(1, "live-galaxy-x4-build-2", ["live-galaxy-observation-v2"])
}

#[test]
fn compatible_reconnect_advances_only_bridge_generation() {
    let initial = SessionState::new(1);
    let admitted = initial.admit_hello(compatible_hello());
    let reconnected = admitted.reconnect();

    assert_eq!(admitted.decision(), CapabilityDecision::Compatible);
    assert_eq!(admitted.generation(), SessionGeneration::new(1));
    assert_eq!(reconnected.decision(), CapabilityDecision::Compatible);
    assert_eq!(reconnected.generation(), SessionGeneration::new(2));
    assert!(reconnected.restart_requirement().is_none());
}

#[test]
fn incompatible_hello_is_terminal_until_x4_restarts() {
    let incompatible = SessionState::new(1).admit_hello(SessionHello::new(
        2,
        "live-galaxy-x4-build-2",
        ["live-galaxy-observation-v2"],
    ));

    assert_eq!(
        incompatible.restart_requirement(),
        Some(RestartRequirement::ProtocolMajorMismatch)
    );
    assert_eq!(incompatible.reconnect(), incompatible);
    assert_eq!(incompatible.admit_hello(compatible_hello()), incompatible);
}

#[test]
fn missing_capability_and_game_build_mismatch_name_exact_restart_owner() {
    let missing_capability = SessionState::new(1).admit_hello(SessionHello::new(
        1,
        "live-galaxy-x4-build-2",
        ["other-capability"],
    ));
    let build_mismatch = SessionState::new(1).admit_hello(SessionHello::new(
        1,
        "live-galaxy-x4-build-1",
        ["live-galaxy-observation-v2"],
    ));

    assert_eq!(
        missing_capability.restart_requirement(),
        Some(RestartRequirement::MissingRequiredCapability)
    );
    assert_eq!(
        build_mismatch.restart_requirement(),
        Some(RestartRequirement::GameBuildMismatch)
    );
}

#[test]
fn compatible_session_rejects_stale_sequence_numbers() {
    let session = SessionState::new(1).admit_hello(compatible_hello());
    let Some(advanced) = session.accept_sequence(SequenceNumber::new(1)) else {
        panic!("first sequence is accepted");
    };

    assert_eq!(advanced.accept_sequence(SequenceNumber::new(1)), None);
    assert_eq!(advanced.accept_sequence(SequenceNumber::new(0)), None);
}
