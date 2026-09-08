#![expect(
    clippy::panic,
    reason = "bounded integration watchdogs fail immediately"
)]

use std::path::{Path, PathBuf};
use std::process::{Child, Command, Stdio};
use std::sync::atomic::{AtomicU64, Ordering};
use x4_carrier_native::{HandleToken, NativeTransport, TransportPoll, TransportSendOutcome};

static NEXT_DIRECTORY: AtomicU64 = AtomicU64::new(1);

pub fn spawn_bridge(data_dir: &Path, limits: &Path) -> Child {
    Command::new(env!("CARGO_BIN_EXE_x4-bridge"))
        .args(["--data-dir", data_dir.to_str().expect("path is Unicode")])
        .args(["--limits-file", limits.to_str().expect("path is Unicode")])
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .expect("bridge child starts")
}

pub fn wait_for_history(directory: &Path, expected: &str) {
    let path = directory.join("operational-history.jsonl");
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    while std::time::Instant::now() < deadline {
        if std::fs::read_to_string(&path).is_ok_and(|value| value.contains(expected)) {
            return;
        }
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    panic!("watchdog expired waiting for {expected}");
}

pub const fn valid_limits() -> &'static str {
    r#"{"complete_message_bytes":8192,"control_message_bytes":2048,"max_candidate_raw_bytes":4096,"max_candidate_records":32,"max_candidate_batches":16,"max_candidate_work":1000,"max_message_age_millis":60000,"max_message_inactivity_millis":10000,"max_candidates":10,"max_aggregate_bytes":16384,"max_aggregate_records":64,"max_aggregate_batches":64,"max_aggregate_work":4096,"max_publication_records":32,"max_publication_content_bytes":8192,"max_pending_bytes":8192,"max_total_bytes":32768,"max_lifecycle_work":4096,"max_delivery_attempts":4,"max_blockers":16,"reconnect_attempts":2,"reconnect_delay_millis":50,"availability_interval_millis":50}"#
}

pub fn wait_connected(transport: &NativeTransport) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    while !transport.snapshot().connected && std::time::Instant::now() < deadline {
        std::thread::sleep(std::time::Duration::from_millis(10));
    }
    assert!(
        transport.snapshot().connected,
        "connection watchdog expired"
    );
}

pub fn send_data(transport: &NativeTransport, token: HandleToken, bytes: &[u8]) {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    while std::time::Instant::now() < deadline {
        match transport.try_send(token, bytes) {
            TransportSendOutcome::LocalHandoff => return,
            TransportSendOutcome::CapacityUnavailable => std::thread::yield_now(),
            outcome @ TransportSendOutcome::Rejected(_) => panic!("data rejected: {outcome:?}"),
        }
    }
    panic!("data watchdog expired");
}

pub fn await_control(transport: &NativeTransport, token: HandleToken) -> Vec<u8> {
    let deadline = std::time::Instant::now() + std::time::Duration::from_secs(3);
    while std::time::Instant::now() < deadline {
        match transport.poll_control(token, 2_048) {
            TransportPoll::Message(bytes) => return bytes,
            TransportPoll::NoMessage => std::thread::yield_now(),
            outcome => panic!("control rejected: {outcome:?}"),
        }
    }
    panic!("control watchdog expired");
}

pub fn fresh_readback(directory: &Path) {
    let output = Command::new(env!("CARGO_BIN_EXE_x4-bridge"))
        .args([
            "--readback",
            "--data-dir",
            directory.to_str().expect("path"),
        ])
        .args([
            "--section-key",
            "carrier_b_realtime_sample",
            "--section-revision",
            "1",
        ])
        .output()
        .expect("fresh readback process");
    assert!(output.status.success());
    let value: serde_json::Value = serde_json::from_slice(&output.stdout).expect("typed JSON");
    assert_eq!(value["section_key"], "carrier_b_realtime_sample");
    assert_eq!(value["section_revision"], 1);
    assert_eq!(value["records"].as_array().map(Vec::len), Some(1));
    let record = &value["records"][0];
    assert_eq!(record["record_id"], "record:sample:1");
    assert_eq!(record["entity_id"], "x4:runtime:realtime_clock");
    assert_eq!(record["observation_version"], 1);
    assert_eq!(record["content"], "123.0");
    assert_eq!(value["receipt"]["ordinal"], 1);
    assert!(value["receipt"]["accepted_at"].as_u64().is_some());
}

pub struct TempDirectory(PathBuf);

impl TempDirectory {
    pub fn new(label: &str) -> Self {
        let ordinal = NEXT_DIRECTORY.fetch_add(1, Ordering::Relaxed);
        let path = std::env::temp_dir().join(format!(
            "live-galaxy-production-{label}-{}-{ordinal}",
            std::process::id()
        ));
        std::fs::create_dir_all(&path).expect("temporary directory");
        Self(path)
    }

    pub fn path(&self) -> &Path {
        &self.0
    }
}

impl Drop for TempDirectory {
    fn drop(&mut self) {
        let _ = std::fs::remove_dir_all(&self.0);
    }
}
