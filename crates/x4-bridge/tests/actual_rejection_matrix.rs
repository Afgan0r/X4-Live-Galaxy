#![expect(
    clippy::expect_used,
    reason = "invalid integration fixtures fail immediately"
)]

#[path = "carrier_b_support/mod.rs"]
mod carrier_b_support;
#[path = "production_startup/support.rs"]
#[expect(dead_code, reason = "shared integration support has extra helpers")]
mod startup_support;

use observation_domain::TransportEpoch;
use observation_ingest::{
    CarrierControl, CarrierIdentity, ControlBody, HandshakeBody, encode_carrier_control,
};
use startup_support::{
    TempDirectory, await_control, send_data, spawn_bridge, valid_limits, wait_connected,
    wait_for_history,
};
use x4_bridge::PIPE_ENDPOINT;
use x4_carrier_native::{HandleToken, NativeTransport, TransportConfig};

#[test]
fn actual_pipe_rejects_decoder_matrix_and_recovers_bridge() {
    let start = String::from_utf8(carrier_b_support::start_bytes("carrier_b_realtime_sample"))
        .expect("fixture");
    for (label, bytes) in [
        (
            "version-one",
            start.replacen("\"contract_version\":2", "\"contract_version\":1", 1),
        ),
        (
            "version-unknown",
            start.replacen("\"contract_version\":2", "\"contract_version\":99", 1),
        ),
        (
            "decoded-bytes",
            String::from_utf8(carrier_b_support::completion_bytes(
                "carrier_b_realtime_sample",
            ))
            .expect("fixture")
            .replacen("\"raw_bytes\":0", "\"raw_bytes\":0,\"decoded_bytes\":0", 1),
        ),
    ] {
        reject_actual(label, bytes.as_bytes());
    }
    bridge_restart_accepts_a_fresh_bootstrap();
}

fn reject_actual(label: &str, bytes: &[u8]) {
    let mut harness = Harness::start(label);
    harness.bootstrap(label);
    send_data(&harness.transport, harness.token, bytes);
    wait_for_history(harness.directory.path(), "message-decode");
    harness.stop();
}

fn bridge_restart_accepts_a_fresh_bootstrap() {
    let mut harness = Harness::start("bridge-restart");
    harness.bootstrap("before-restart");
    harness.child.kill().expect("bridge killed");
    harness.child.wait().expect("bridge reaped");
    std::thread::sleep(std::time::Duration::from_millis(50));
    harness.child = spawn_bridge(
        harness.directory.path(),
        &harness.directory.path().join("limits.json"),
    );
    wait_connected(&harness.transport);
    harness.bootstrap("after-restart");
    harness.stop();
}

struct Harness {
    directory: TempDirectory,
    transport: NativeTransport,
    token: HandleToken,
    child: std::process::Child,
}

impl Harness {
    fn start(label: &str) -> Self {
        let directory = TempDirectory::new(label);
        let limits = directory.path().join("limits.json");
        std::fs::write(&limits, valid_limits()).expect("limits");
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

    fn bootstrap(&self, label: &str) {
        let identity = CarrierIdentity {
            session_id: format!("session-{label}"),
            producer_incarnation: "producer:1".to_owned(),
            epoch: TransportEpoch::new(1).expect("epoch"),
        };
        let bytes = encode_carrier_control(
            &CarrierControl {
                identity,
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
    }

    fn stop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = self.transport.request_close(self.token);
        assert!(
            self.transport
                .wait_closed_for_test(std::time::Duration::from_secs(2))
        );
    }
}

impl Drop for Harness {
    fn drop(&mut self) {
        let _ = self.child.kill();
        let _ = self.child.wait();
        let _ = self.transport.request_close(self.token);
    }
}
