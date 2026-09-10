#![expect(
    clippy::panic,
    reason = "bounded integration watchdogs fail immediately"
)]

use observation_domain::TransportEpoch;
use observation_ingest::{
    CarrierControl, CarrierIdentity, ControlBody, complete_message_digest,
    decode_carrier_bootstrap, decode_carrier_control, encode_carrier_control,
};
use x4_bridge::{CarrierCodecError, PIPE_ENDPOINT};
use x4_carrier_native::{HandleToken, NativeTransport, TransportConfig};

use super::startup_support::{
    TempDirectory, await_control, fresh_readback, send_data, spawn_bridge, valid_limits,
    wait_connected, wait_for_history,
};

#[path = "actual_pipe/sample.rs"]
mod sample;

#[test]
fn bootstrap_is_handshake_only_and_message_digest_is_byte_exact() {
    let expected = identity("bootstrap");
    let bootstrap = control(&expected, ControlBody::Handshake(exact_handshake()));
    assert_eq!(decode_carrier_bootstrap(&bootstrap, 2_048), Ok(expected));
    for stale in [1, 2] {
        let text = String::from_utf8(bootstrap.clone()).expect("bootstrap is UTF-8");
        let stale_text = text.replacen(
            "\"control_version\":3",
            &format!("\"control_version\":{stale}"),
            1,
        );
        assert_eq!(
            decode_carrier_bootstrap(stale_text.as_bytes(), 2_048),
            Err(CarrierCodecError::RestartRequired)
        );
    }
    assert_eq!(
        decode_carrier_bootstrap(
            br#"{"control_version":3,"kind":"health","session":"s","incarnation":"i","epoch":1,"body":{"native_abi":2,"envelope_contract":2,"schema_version":1,"policy_version":2,"canonicalization_version":3,"digest_version":1}}"#,
            2_048,
        ),
        Err(CarrierCodecError::UnknownKind)
    );
    assert_eq!(
        complete_message_digest(b"abc"),
        [
            0xba, 0x78, 0x16, 0xbf, 0x8f, 0x01, 0xcf, 0xea, 0x41, 0x41, 0x40, 0xde, 0x5d, 0xae,
            0x22, 0x23, 0xb0, 0x03, 0x61, 0xa3, 0x96, 0x17, 0x7a, 0x9c, 0xb4, 0x10, 0xff, 0x61,
            0xf2, 0x00, 0x15, 0xad,
        ]
    );
    assert_ne!(
        complete_message_digest(b"abc"),
        complete_message_digest(b"abd")
    );
}

#[test]
fn both_startup_orders_publish_actual_three_message_sample() {
    run_actual_sample("native-first", true);
    run_actual_sample("bridge-first", false);
}

fn run_actual_sample(label: &str, native_first: bool) {
    let directory = TempDirectory::new(label);
    let limits = directory.path().join("limits.json");
    std::fs::write(&limits, valid_limits()).expect("limits written");
    let config = TransportConfig {
        pipe_name: PIPE_ENDPOINT.to_owned(),
        max_data_message_bytes: 8_192,
        max_control_message_bytes: 2_048,
    };
    let token = HandleToken {
        generation: 1,
        slot: 1,
    };
    let (transport, mut child) = if native_first {
        let transport = NativeTransport::start(config, token).expect("native server");
        (transport, spawn_bridge(directory.path(), &limits))
    } else {
        let child = spawn_bridge(directory.path(), &limits);
        wait_for_history(directory.path(), "peer-absent");
        (
            NativeTransport::start(config, token).expect("native server"),
            child,
        )
    };
    wait_connected(&transport);
    let identity = identity(label);
    send_data(
        &transport,
        token,
        &control(&identity, ControlBody::Handshake(exact_handshake())),
    );
    for expected in ["handshake", "collection_intent", "demand"] {
        let value = decode_carrier_control(&await_control(&transport, token), &identity, 2_048)
            .expect("bound control");
        if let ControlBody::CollectionIntent(intent) = &value.body {
            assert_eq!(intent.next_revision, 1);
        }
        assert!(matches!(
            (expected, value.body),
            ("handshake", ControlBody::Handshake(_))
                | ("collection_intent", ControlBody::CollectionIntent(_))
                | ("demand", ControlBody::Demand(_))
        ));
    }
    let mut dispositions = Vec::new();
    for message in sample::messages() {
        send_data(&transport, token, &message);
        let value = decode_carrier_control(&await_control(&transport, token), &identity, 2_048)
            .expect("disposition");
        let ControlBody::Disposition(value) = value.body else {
            panic!("disposition expected");
        };
        dispositions.push(value.disposition);
    }
    child.kill().expect("runner-owned child terminates");
    child.wait().expect("runner-owned child reaped");
    assert_eq!(dispositions, ["received", "received", "committed"]);
    wait_for_history(directory.path(), "\"state\":\"committed\"");
    fresh_readback(directory.path());
    let _ = transport.request_close(token);
    assert!(transport.wait_closed_for_test(std::time::Duration::from_secs(2)));
}

fn identity(label: &str) -> CarrierIdentity {
    CarrierIdentity {
        session_id: format!("session-{label}"),
        producer_incarnation: "producer:1".to_owned(),
        epoch: TransportEpoch::new(1).expect("epoch"),
    }
}

const fn exact_handshake() -> observation_ingest::HandshakeBody {
    observation_ingest::HandshakeBody {
        native_abi: 2,
        envelope_contract: 2,
        schema_version: 1,
        policy_version: 2,
        canonicalization_version: 3,
        digest_version: 1,
    }
}

fn control(identity: &CarrierIdentity, body: ControlBody) -> Vec<u8> {
    encode_carrier_control(
        &CarrierControl {
            identity: identity.clone(),
            body,
        },
        2_048,
    )
    .expect("control encodes")
}
