use super::*;

#[test]
fn repeated_events_rotate_with_bounded_gap_status() {
    let path = std::env::temp_dir().join(format!("history-bound-{}", std::process::id()));
    let mut history = OperationalHistory::open(&path).expect("history");
    for index in 0..2_000 {
        let _ = history.record("waiting", &format!("reason-{index}"));
    }
    assert!(
        std::fs::metadata(path.join("operational-history.jsonl"))
            .is_ok_and(|value| value.len() <= MAX_HISTORY_BYTES as u64)
    );
    assert!(path.join("operational-history.jsonl.1").is_file());
    for _ in 0..32 {
        let _ = history.record("waiting", "same");
    }
    assert!(history.suppressed > 0);
    history.sink = None;
    assert!(!history.record("degraded", "forced-test-gap"));
    assert_eq!(history.history_gap_count(), 1);
    assert!(
        std::fs::read_to_string(path.join("operational-status.json"))
            .is_ok_and(|value| value.contains("\"history_gap_count\":1"))
    );
    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn post_start_status_sink_failure_is_visible_in_bounded_history() {
    let path = std::env::temp_dir().join(format!("status-gap-{}", std::process::id()));
    let mut history = OperationalHistory::open(&path).expect("history");
    let status = path.join("operational-status.json");
    std::fs::remove_file(&status).expect("replace status file");
    std::fs::create_dir(&status).expect("block status-file writes");

    assert!(!history.record("collection", "first-status-gap"));
    assert!(!history.record("collection", "second-status-gap"));
    let events = std::fs::read_to_string(path.join("operational-history.jsonl"))
        .expect("history remains readable");
    assert!(events.contains("\"status_gap_count\":1"));
    assert!(events.len() <= MAX_HISTORY_BYTES);

    let _ = std::fs::remove_dir_all(path);
}

#[test]
fn duplicate_suppression_never_crosses_session_or_message_identity() {
    let path = std::env::temp_dir().join(format!("history-identity-{}", std::process::id()));
    let mut history = OperationalHistory::open(&path).expect("history");
    history.bind_session("session-a", 1);
    history.bind_message("message-a", "section", 1);
    for _ in 0..32 {
        let _ = history.record("collection", "received");
    }
    let suppressed = history.suppressed;
    assert!(suppressed > 0);

    history.bind_session("session-b", 2);
    history.bind_message("message-b", "section", 2);
    assert!(history.record("collection", "received"));

    let events = std::fs::read_to_string(path.join("operational-history.jsonl"))
        .expect("history remains readable");
    assert!(events.contains("\"session\":\"session-b\",\"epoch\":2"));
    assert!(events.contains("\"message\":\"message-b\",\"section\":\"section\",\"revision\":2"));
    assert!(events.contains(&format!("\"suppressed_before\":{suppressed}")));

    let _ = std::fs::remove_dir_all(path);
}
