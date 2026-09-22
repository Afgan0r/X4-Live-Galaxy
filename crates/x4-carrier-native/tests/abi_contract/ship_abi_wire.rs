#![expect(
    clippy::unwrap_used,
    clippy::panic,
    reason = "real ABI fixtures fail immediately when their contract is violated"
)]

use observation_domain::CompleteMessage;
use observation_ingest::{
    CarrierControl, CollectionIntentBody, ControlBody, DemandBody, decode_carrier_bootstrap,
    decode_complete_message, encode_carrier_control,
};
use std::fmt::Write as _;
use std::time::{Duration, Instant};
use x4_carrier_native::{BridgePeer, TransportConfig};
pub(super) fn connect() -> BridgePeer {
    let deadline = Instant::now() + Duration::from_secs(5);
    loop {
        if let Ok(peer) = BridgePeer::connect(
            &TransportConfig {
                pipe_name: r"\\.\pipe\live_galaxy".to_owned(),
                max_data_message_bytes: 4096,
                max_control_message_bytes: 512,
            },
            Duration::from_secs(5),
        ) {
            return peer;
        }
        assert!(Instant::now() < deadline, "owned Lua pipe startup watchdog");
        std::thread::yield_now();
    }
}
pub(super) fn send(
    peer: &mut BridgePeer,
    identity: &observation_ingest::CarrierIdentity,
    body: ControlBody,
) {
    peer.send_control(
        &encode_carrier_control(
            &CarrierControl {
                identity: identity.clone(),
                body,
            },
            512,
        )
        .unwrap(),
    )
    .unwrap();
}
pub(super) fn intent(key: &str) -> ControlBody {
    ControlBody::CollectionIntent(CollectionIntentBody {
        section_key: key.to_owned(),
        next_revision: 7,
        max_records: 129,
        max_raw_bytes: 512,
        max_work: 129,
    })
}
pub(super) fn qualify(peer: &mut BridgePeer, key: &str) -> observation_ingest::CarrierIdentity {
    let identity = decode_carrier_bootstrap(&peer.receive(512).unwrap(), 512).unwrap();
    send(peer, &identity, super_handshake());
    send(peer, &identity, intent(key));
    send(
        peer,
        &identity,
        ControlBody::Demand(DemandBody { credit: 1 }),
    );
    identity
}
pub(super) fn super_handshake() -> ControlBody {
    ControlBody::Handshake(observation_ingest::HandshakeBody {
        native_abi: 2,
        envelope_contract: 2,
        schema_version: 1,
        policy_version: 2,
        canonicalization_version: 3,
        digest_version: 1,
    })
}
pub(super) fn receive(
    peer: &mut BridgePeer,
    identity: &observation_ingest::CarrierIdentity,
    result: &str,
) -> CompleteMessage {
    let bytes = peer.receive(4096).unwrap();
    let message = decode_complete_message(&bytes, 4096).unwrap();
    let (id, key, revision) = match &message {
        CompleteMessage::SectionStart(v) => (
            format!(
                "message:start:{}:{}",
                v.section_key.as_str(),
                v.section_revision.get()
            ),
            v.section_key.clone(),
            v.section_revision,
        ),
        CompleteMessage::ImmutableBatch(v) => (
            v.batch_id.as_str().to_owned(),
            v.section_key.clone(),
            v.section_revision,
        ),
        CompleteMessage::SectionCompletion(v) => (
            format!(
                "message:complete:{}:{}",
                v.section_key.as_str(),
                v.section_revision.get()
            ),
            v.section_key.clone(),
            v.section_revision,
        ),
        CompleteMessage::Control(_) => panic!("data required"),
    };
    let digest = observation_ingest::complete_message_digest(&bytes)
        .iter()
        .fold(String::with_capacity(64), |mut text, byte| {
            write!(text, "{byte:02x}").unwrap();
            text
        });
    send(
        peer,
        identity,
        ControlBody::Disposition(observation_ingest::DispositionBody {
            message_id: id,
            section_key: key.as_str().to_owned(),
            section_revision: revision.get(),
            message_digest: digest,
            disposition: result.to_owned(),
        }),
    );
    message
}
