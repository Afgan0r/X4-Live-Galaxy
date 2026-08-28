use x4_bridge::{CapabilityDecision, SessionHello, SessionState};

fn hello() -> SessionHello {
    SessionHello::new(1, "live-galaxy-x4-build-1", ["live-galaxy-observation-v1"])
}

#[test]
fn compatible_reconnect_advances_generation_without_mutating_acceptance() {
    let first = SessionState::new(1).admit_hello(hello());
    let second = first.reconnect();

    assert_eq!(first.decision(), CapabilityDecision::Compatible);
    assert_eq!(second.decision(), CapabilityDecision::Compatible);
    assert_ne!(first.generation(), second.generation());
}

#[test]
fn incompatible_reconnect_stays_terminal() {
    let rejected = SessionState::new(1).admit_hello(SessionHello::new(
        2,
        "live-galaxy-x4-build-1",
        ["live-galaxy-observation-v1"],
    ));
    assert_eq!(rejected.reconnect(), rejected);
}
