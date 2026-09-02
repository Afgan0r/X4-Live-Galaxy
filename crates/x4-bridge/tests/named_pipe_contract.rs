use x4_bridge::{PIPE_ENDPOINT, PipeDisposition, PipeServer, is_telemetry_only};

#[path = "named_pipe_contract/accept_contract.rs"]
mod accept_contract;
#[path = "named_pipe_contract/deferred_publication.rs"]
mod deferred_publication;
#[path = "named_pipe_contract/fixtures.rs"]
mod fixtures;

use fixtures::{
    HEALTH, HEARTBEAT, HELLO, MARKER, MARKER_CONFIRMATION, OBSERVATION, observation,
    station_observation,
};

#[test]
fn project_pipe_identity_and_typed_batch_admission_are_aligned() {
    let mut server = PipeServer::new();

    assert_eq!(PIPE_ENDPOINT, r"\\.\pipe\live_galaxy");
    for frame in [
        HELLO,
        HEARTBEAT,
        HEALTH,
        OBSERVATION,
        MARKER,
        MARKER_CONFIRMATION,
    ] {
        assert_eq!(server.admit_message(frame), PipeDisposition::Accepted);
    }
    assert_eq!(server.snapshot().entity_ids(), vec!["sector:argon_prime"]);
    assert_eq!(server.snapshot().entity_ids(), vec!["sector:argon_prime"]);
    assert!(is_telemetry_only());
}

#[test]
fn oversize_or_malformed_messages_fail_closed() {
    let mut server = PipeServer::new();

    assert_eq!(server.admit_message(HELLO), PipeDisposition::Accepted);
    assert_eq!(server.admit_message("not json"), PipeDisposition::Rejected);
    assert_eq!(
        server.admit_message(&"x".repeat(2_049)),
        PipeDisposition::Rejected
    );
}

#[test]
fn disconnect_or_conflicting_marker_discards_only_the_pending_snapshot() {
    let mut server = PipeServer::new();

    assert_eq!(server.admit_message(HELLO), PipeDisposition::Accepted);
    assert_eq!(server.admit_message(OBSERVATION), PipeDisposition::Accepted);
    server.discard_pending();
    assert_eq!(server.admit_message(MARKER), PipeDisposition::Rejected);
    assert_eq!(server.snapshot().entity_ids(), Vec::<String>::new());
    assert_eq!(
        server.admit_message(&observation(1, 5)),
        PipeDisposition::Accepted
    );
    assert_eq!(
        server.admit_message(r#"{"type":"complete_marker","scope":"runtime:sectors","version":2}"#),
        PipeDisposition::Rejected
    );
    assert_eq!(server.snapshot().entity_ids(), Vec::<String>::new());
}

#[test]
fn station_frames_remain_pending_until_marker_confirmation() {
    let mut server = PipeServer::new();
    let first = station_observation(10, "sector:argon_prime", 1);
    let second = station_observation(20, "sector:second_contact", 2);
    let marker = r#"{"type":"complete_marker","scope":"runtime:sectors","version":2,"generation":1,"sequence":3}"#;

    assert_eq!(server.admit_message(HELLO), PipeDisposition::Accepted);
    assert_eq!(server.admit_message(&first), PipeDisposition::Accepted);
    assert_eq!(server.admit_message(&second), PipeDisposition::Accepted);
    assert_eq!(server.snapshot().entity_ids(), Vec::<String>::new());
    assert_eq!(server.admit_message(marker), PipeDisposition::Accepted);
    assert_eq!(server.snapshot().entity_ids(), Vec::<String>::new());
    assert_eq!(
        server.admit_message(MARKER_CONFIRMATION),
        PipeDisposition::Accepted
    );
    assert_eq!(
        server.snapshot().entity_ids(),
        vec!["asset:station:10", "asset:station:20"]
    );
}

#[test]
fn session_rejects_stale_generation_replay_and_terminal_hello() {
    let mut server = PipeServer::new();

    assert_eq!(server.admit_message(HELLO), PipeDisposition::Accepted);
    assert_eq!(server.admit_message(HEARTBEAT), PipeDisposition::Accepted);
    assert_eq!(server.admit_message(HEARTBEAT), PipeDisposition::Rejected);
    assert_eq!(server.admit_message(HELLO), PipeDisposition::Rejected);
    assert_eq!(server.snapshot().entity_ids(), Vec::<String>::new());
    let mut mismatch = PipeServer::new();
    assert_eq!(mismatch.admit_message(r#"{"type":"hello","protocol_major":1,"game_build":"live-galaxy-x4-build-1","capabilities":["live-galaxy-observation-v1"],"generation":1}"#), PipeDisposition::Rejected);
}

#[test]
fn higher_generation_reconnect_preserves_completed_projection() {
    let mut server = PipeServer::new();
    for frame in [
        HELLO,
        HEARTBEAT,
        HEALTH,
        OBSERVATION,
        MARKER,
        MARKER_CONFIRMATION,
    ] {
        assert_eq!(server.admit_message(frame), PipeDisposition::Accepted);
    }
    assert_eq!(server.admit_message(r#"{"type":"hello","protocol_major":1,"game_build":"live-galaxy-x4-build-2","capabilities":["live-galaxy-observation-v2"],"generation":2}"#), PipeDisposition::Accepted);
    assert_eq!(server.admit_message(r#"{"type":"heartbeat","scope":"runtime:sectors","version":1,"generation":2,"sequence":1}"#), PipeDisposition::Accepted);
    assert_eq!(server.admit_message(HEALTH), PipeDisposition::Rejected);
    assert_eq!(server.snapshot().entity_ids(), vec!["sector:argon_prime"]);
}

#[test]
fn unconfirmed_marker_then_disconnect_preserves_the_prior_projection() {
    let mut server = PipeServer::new();
    let next_observation = station_observation(10, "sector:argon_prime", 6);
    let ambiguous_marker = r#"{"type":"complete_marker","scope":"runtime:sectors","version":2,"generation":1,"sequence":7}"#;
    let reconnect = r#"{"type":"hello","protocol_major":1,"game_build":"live-galaxy-x4-build-2","capabilities":["live-galaxy-observation-v2"],"generation":2}"#;
    let post_reconnect_heartbeat =
        r#"{"type":"heartbeat","scope":"runtime:sectors","version":2,"generation":2,"sequence":1}"#;

    for frame in [
        HELLO,
        HEARTBEAT,
        HEALTH,
        OBSERVATION,
        MARKER,
        MARKER_CONFIRMATION,
    ] {
        assert_eq!(server.admit_message(frame), PipeDisposition::Accepted);
    }
    assert_eq!(server.snapshot().entity_ids(), vec!["sector:argon_prime"]);
    assert_eq!(
        server.admit_message(&next_observation),
        PipeDisposition::Accepted
    );
    assert_eq!(
        server.admit_message(ambiguous_marker),
        PipeDisposition::Accepted
    );
    assert_eq!(server.snapshot().entity_ids(), vec!["sector:argon_prime"]);

    server.discard_pending();
    assert_eq!(server.admit_message(reconnect), PipeDisposition::Accepted);
    assert_eq!(
        server.admit_message(post_reconnect_heartbeat),
        PipeDisposition::Accepted
    );
    assert_eq!(server.snapshot().entity_ids(), vec!["sector:argon_prime"]);
}

#[test]
fn completed_cycles_release_ingress_capacity_without_replaying_sequences() {
    let mut server = PipeServer::new();
    assert_eq!(server.admit_message(HELLO), PipeDisposition::Accepted);
    for version in 1..=80_u64 {
        let sequence = version * 2;
        let observation = observation(version, sequence);
        let marker = format!(
            "{{\"type\":\"complete_marker\",\"scope\":\"runtime:sectors\",\"version\":{version},\"generation\":1,\"sequence\":{}}}",
            sequence + 1
        );
        assert_eq!(
            server.admit_message(&observation),
            PipeDisposition::Accepted
        );
        assert_eq!(server.admit_message(&marker), PipeDisposition::Accepted);
    }
    assert_eq!(server.snapshot().entity_ids(), vec!["sector:argon_prime"]);
}
