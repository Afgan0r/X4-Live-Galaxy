use x4_bridge::{
    AcceptAttempt, AcceptDisposition, MAX_CONSECUTIVE_ACCEPT_FAILURES, PIPE_ENDPOINT,
    PipeDisposition, PipeServer, is_telemetry_only,
};

const HELLO: &str = r#"{"type":"hello","protocol_major":1,"game_build":"live-galaxy-x4-build-2","capabilities":["live-galaxy-observation-v2"],"generation":1}"#;
const OBSERVATION: &str = r#"{"type":"observation","scope":"runtime:sectors","entity_id":"sector:argon_prime","version":1,"quality":"fresh","runtime_facts":{"r":"x4_runtime","g":42,"q":"fresh","a":"available","s":[{"i":"sector:argon_prime"}],"x":[{"i":"asset:ship:1","p":"sector:argon_prime"}],"c":[{"i":"capacity:ship:storage","p":"asset:ship:1","v":42}],"o":[{"i":"ownership:ship:1","p":"asset:ship:1","n":"faction:argon"}]},"generation":1,"sequence":3}"#;
const MARKER: &str = r#"{"type":"complete_marker","scope":"runtime:sectors","version":1,"generation":1,"sequence":4}"#;
const HEARTBEAT: &str =
    r#"{"type":"heartbeat","scope":"runtime:sectors","version":1,"generation":1,"sequence":1}"#;
const HEALTH: &str = r#"{"type":"runtime_health","scope":"runtime:sectors","version":1,"status":"available","generation":1,"sequence":2}"#;

fn observation(version: u64, sequence: u64) -> String {
    format!(
        r#"{{"type":"observation","scope":"runtime:sectors","entity_id":"sector:argon_prime","version":{version},"quality":"fresh","runtime_facts":{{"r":"x4_runtime","g":42,"q":"fresh","a":"available","s":[{{"i":"sector:argon_prime"}}],"x":[{{"i":"asset:ship:1","p":"sector:argon_prime"}}],"c":[{{"i":"capacity:ship:storage","p":"asset:ship:1","v":42}}],"o":[{{"i":"ownership:ship:1","p":"asset:ship:1","n":"faction:argon"}}]}},"generation":1,"sequence":{sequence}}}"#
    )
}

fn station_observation(station: u64, sector: &str, sequence: u64) -> String {
    format!(
        r#"{{"type":"observation","scope":"runtime:sectors","entity_id":"asset:station:{station}","version":2,"quality":"fresh","runtime_facts":{{"r":"x4_runtime","q":"fresh","a":"available","s":[{{"i":"{sector}"}}],"x":[{{"i":"asset:station:{station}","p":"{sector}"}}],"c":[{{"i":"capacity:station:{station}","p":"asset:station:{station}","v":42}}],"o":[{{"i":"ownership:station:{station}","p":"asset:station:{station}","n":"faction:argon"}}]}},"generation":1,"sequence":{sequence}}}"#
    )
}

#[test]
fn project_pipe_identity_and_typed_batch_admission_are_aligned() {
    let mut server = PipeServer::new();

    assert_eq!(PIPE_ENDPOINT, r"\\.\pipe\live_galaxy");
    for frame in [HELLO, HEARTBEAT, HEALTH, OBSERVATION, MARKER] {
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
fn station_frames_remain_pending_until_one_matching_marker() {
    let mut server = PipeServer::new();
    let first = station_observation(10, "sector:argon_prime", 1);
    let second = station_observation(20, "sector:second_contact", 2);
    let marker = r#"{"type":"complete_marker","scope":"runtime:sectors","version":2,"generation":1,"sequence":3}"#;

    assert_eq!(server.admit_message(HELLO), PipeDisposition::Accepted);
    assert_eq!(server.admit_message(&first), PipeDisposition::Accepted);
    assert_eq!(server.admit_message(&second), PipeDisposition::Accepted);
    assert_eq!(server.snapshot().entity_ids(), Vec::<String>::new());
    assert_eq!(server.admit_message(marker), PipeDisposition::Accepted);
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
    for frame in [HELLO, HEARTBEAT, HEALTH, OBSERVATION, MARKER] {
        assert_eq!(server.admit_message(frame), PipeDisposition::Accepted);
    }
    assert_eq!(server.admit_message(r#"{"type":"hello","protocol_major":1,"game_build":"live-galaxy-x4-build-2","capabilities":["live-galaxy-observation-v2"],"generation":2}"#), PipeDisposition::Accepted);
    assert_eq!(server.admit_message(r#"{"type":"heartbeat","scope":"runtime:sectors","version":1,"generation":2,"sequence":1}"#), PipeDisposition::Accepted);
    assert_eq!(server.admit_message(HEALTH), PipeDisposition::Rejected);
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

#[test]
fn transient_accept_error_retries_before_a_replacement_client() {
    let mut server = PipeServer::new();

    assert_eq!(server.admit_message(HELLO), PipeDisposition::Accepted);
    assert_eq!(server.admit_message(OBSERVATION), PipeDisposition::Accepted);
    for _ in 1..MAX_CONSECUTIVE_ACCEPT_FAILURES {
        assert_eq!(
            server.record_accept(AcceptAttempt::TransientFailure),
            AcceptDisposition::RetryAccept
        );
    }
    assert_eq!(
        server.record_accept(AcceptAttempt::TransientFailure),
        AcceptDisposition::RetryAcceptDegraded { delay_millis: 100 }
    );
    assert_eq!(
        server.record_accept(AcceptAttempt::TransientFailure),
        AcceptDisposition::RetryAcceptDegraded { delay_millis: 200 }
    );
    for expected_delay in [400, 800, 1_000, 1_000] {
        assert_eq!(
            server.record_accept(AcceptAttempt::TransientFailure),
            AcceptDisposition::RetryAcceptDegraded {
                delay_millis: expected_delay
            }
        );
    }
    assert_eq!(
        server.record_accept(AcceptAttempt::ClientAccepted),
        AcceptDisposition::ServeClient
    );
    assert_eq!(server.snapshot().entity_ids(), Vec::<String>::new());
}
