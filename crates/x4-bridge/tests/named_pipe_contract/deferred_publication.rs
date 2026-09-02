// Historical RED specifications; reconcile with Phase 05.3 before enabling.
use super::fixtures::{HEALTH, HEARTBEAT, HELLO, MARKER, OBSERVATION, station_observation};
use x4_bridge::{PIPE_ENDPOINT, PipeDisposition, PipeServer, is_telemetry_only};

#[test]
#[ignore = "Deferred 05.1-09 RED contract; reconcile in Phase 05.3 per HANDOFF"]
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
#[ignore = "Deferred 05.1-09 RED contract; reconcile in Phase 05.3 per HANDOFF"]
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
#[ignore = "Deferred 05.1-09 RED contract; reconcile in Phase 05.3 per HANDOFF"]
fn marker_admits_129_contiguous_station_frames_without_an_aggregate_cap() {
    let mut server = PipeServer::new();

    assert_eq!(server.admit_message(HELLO), PipeDisposition::Accepted);
    for offset in 0..129_u64 {
        let station = 1_000 + offset;
        let frame = station_observation(station, "sector:argon_prime", offset + 1);
        assert_eq!(server.admit_message(&frame), PipeDisposition::Accepted);
    }
    let marker = r#"{"type":"complete_marker","scope":"runtime:sectors","version":2,"generation":1,"sequence":130}"#;
    assert_eq!(server.admit_message(marker), PipeDisposition::Accepted);
    assert_eq!(server.snapshot().entity_ids().len(), 129);
}

#[test]
#[ignore = "Deferred 05.1-09 RED contract; reconcile in Phase 05.3 per HANDOFF"]
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
#[ignore = "Deferred 05.1-09 RED contract; reconcile in Phase 05.3 per HANDOFF"]
fn delivered_marker_then_disconnect_preserves_the_completed_projection() {
    let mut server = PipeServer::new();
    let next_observation = station_observation(10, "sector:argon_prime", 5);
    let admitted_marker = r#"{"type":"complete_marker","scope":"runtime:sectors","version":2,"generation":1,"sequence":6}"#;
    let reconnect = r#"{"type":"hello","protocol_major":1,"game_build":"live-galaxy-x4-build-2","capabilities":["live-galaxy-observation-v2"],"generation":2}"#;
    let post_reconnect_heartbeat =
        r#"{"type":"heartbeat","scope":"runtime:sectors","version":2,"generation":2,"sequence":1}"#;

    for frame in [HELLO, HEARTBEAT, HEALTH, OBSERVATION, MARKER] {
        assert_eq!(server.admit_message(frame), PipeDisposition::Accepted);
    }
    assert_eq!(server.snapshot().entity_ids(), vec!["sector:argon_prime"]);
    assert_eq!(
        server.admit_message(&next_observation),
        PipeDisposition::Accepted
    );
    assert_eq!(
        server.admit_message(admitted_marker),
        PipeDisposition::Accepted
    );
    assert_eq!(server.snapshot().entity_ids(), vec!["asset:station:10"]);

    server.discard_pending();
    assert_eq!(server.admit_message(reconnect), PipeDisposition::Accepted);
    assert_eq!(
        server.admit_message(post_reconnect_heartbeat),
        PipeDisposition::Accepted
    );
    assert_eq!(server.snapshot().entity_ids(), vec!["asset:station:10"]);
}
