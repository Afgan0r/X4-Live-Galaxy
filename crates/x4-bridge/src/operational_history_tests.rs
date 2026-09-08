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
