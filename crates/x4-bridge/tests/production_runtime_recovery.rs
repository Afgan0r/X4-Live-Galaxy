#![expect(
    clippy::expect_used,
    reason = "invalid integration fixtures fail immediately"
)]

#[path = "production_runtime_recovery/bootstrap_reconnect.rs"]
mod bootstrap_reconnect;
#[path = "carrier_b_support/mod.rs"]
mod carrier_b_support;
#[path = "production_startup/support.rs"]
#[expect(
    dead_code,
    reason = "shared integration support includes fixture readback"
)]
mod startup_support;

use std::time::Duration;

use observation_domain::TransportEpoch;
use observation_ingest::{
    CarrierControl, CarrierIdentity, ControlBody, HandshakeBody, decode_carrier_control,
    encode_carrier_control,
};
use startup_support::{
    TempDirectory, await_control, send_data, spawn_bridge, valid_limits, wait_connected,
    wait_for_history,
};
use x4_bridge::PIPE_ENDPOINT;
use x4_carrier_native::{HandleToken, NativeTransport, TransportConfig};

static TEST_LOCK: std::sync::Mutex<()> = std::sync::Mutex::new(());

#[test]
fn long_lived_and_stalled_sessions_keep_distinct_deadlines() {
    let _guard = TEST_LOCK.lock().expect("test lock");
    long_lived_session_accepts_late_transport_progress();
    stalled_candidate_expires_before_peer_disconnect();
}

fn long_lived_session_accepts_late_transport_progress() {
    let limits = valid_limits()
        .replace(
            "\"max_message_age_millis\":60000",
            "\"max_message_age_millis\":5",
        )
        .replace(
            "\"max_message_inactivity_millis\":10000",
            "\"max_message_inactivity_millis\":500",
        );
    let mut harness = Harness::start("late-progress", &limits);
    harness.bootstrap("late-progress");
    std::thread::sleep(Duration::from_millis(20));
    send_data(&harness.transport, harness.token, b"not-a-message");
    wait_for_history(harness.directory.path(), "message-decode");
    let history =
        std::fs::read_to_string(harness.directory.path().join("operational-history.jsonl"))
            .expect("history");
    assert!(!history.contains("message-expired"));
    harness.stop();
}

fn stalled_candidate_expires_before_peer_disconnect() {
    let limits = valid_limits()
        .replace(
            "\"max_message_age_millis\":60000",
            "\"max_message_age_millis\":10",
        )
        .replace(
            "\"max_message_inactivity_millis\":10000",
            "\"max_message_inactivity_millis\":100",
        );
    let mut harness = Harness::start("stalled-candidate", &limits);
    let identity = harness.bootstrap("stalled-candidate");
    send_data(
        &harness.transport,
        harness.token,
        &carrier_b_support::start_bytes("carrier_b_realtime_sample"),
    );
    let disposition = decode_carrier_control(
        &await_control(&harness.transport, harness.token),
        &identity,
        2_048,
    )
    .expect("start disposition");
    assert!(matches!(disposition.body, ControlBody::Disposition(_)));
    wait_for_history(harness.directory.path(), "candidate-expired");
    wait_for_history(harness.directory.path(), "peer-inactive");
    harness.stop();
}

struct Harness {
    directory: TempDirectory,
    transport: NativeTransport,
    token: HandleToken,
    child: std::process::Child,
}

impl Harness {
    fn start(label: &str, contents: &str) -> Self {
        let directory = TempDirectory::new(label);
        let limits = directory.path().join("limits.json");
        std::fs::write(&limits, contents).expect("limits");
        let token = HandleToken {
            generation: 1,
            slot: 1,
        };
        let transport = NativeTransport::start(
            TransportConfig {
                pipe_name: PIPE_ENDPOINT.to_owned(),
                max_data_message_bytes: 8_192,
                max_control_message_bytes: 2_048,
            },
            token,
        )
        .expect("native transport");
        let child = spawn_bridge(directory.path(), &limits);
        wait_connected(&transport);
        Self {
            directory,
            transport,
            token,
            child,
        }
    }

    fn bootstrap(&self, label: &str) -> CarrierIdentity {
        let identity = CarrierIdentity {
            session_id: format!("session-{label}"),
            producer_incarnation: "producer:1".to_owned(),
            epoch: TransportEpoch::new(1).expect("epoch"),
        };
        let bytes = encode_carrier_control(
            &CarrierControl {
                identity: identity.clone(),
                body: ControlBody::Handshake(HandshakeBody {
                    native_abi: 2,
                    envelope_contract: 2,
                    schema_version: 1,
                    policy_version: 2,
                    canonicalization_version: 3,
                    digest_version: 1,
                }),
            },
            2_048,
        )
        .expect("bootstrap");
        send_data(&self.transport, self.token, &bytes);
        for _ in 0..3 {
            let _ = await_control(&self.transport, self.token);
        }
        identity
    }

    fn stop(&mut self) {
        self.child.kill().expect("bridge stops");
        self.child.wait().expect("bridge reaped");
        let _ = self.transport.request_close(self.token);
        assert!(self.transport.wait_closed_for_test(Duration::from_secs(2)));
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = self.transport.request_close(self.token);
    }
}
