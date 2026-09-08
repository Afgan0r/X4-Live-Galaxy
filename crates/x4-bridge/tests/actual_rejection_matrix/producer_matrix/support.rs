use observation_domain::TransportEpoch;
use observation_ingest::{CarrierControl, CarrierIdentity, ControlBody, encode_carrier_control};
use x4_carrier_native::{HandleToken, NativeTransport, ProducerSource};

pub fn encode(source: &ProducerSource, body: ControlBody) -> Vec<u8> {
    encode_carrier_control(
        &CarrierControl {
            identity: CarrierIdentity {
                session_id: source.session_id.clone(),
                producer_incarnation: source.producer_incarnation.clone(),
                epoch: TransportEpoch::new(source.transport_epoch).expect("epoch"),
            },
            body,
        },
        2_048,
    )
    .expect("control encode")
}

pub fn hex(bytes: [u8; 32]) -> String {
    use core::fmt::Write as _;
    bytes.iter().fold(String::new(), |mut text, byte| {
        let _ignored = write!(text, "{byte:02x}");
        text
    })
}

pub fn close(transport: &NativeTransport, token: HandleToken) {
    let _ = transport.request_close(token);
    assert!(transport.wait_closed_for_test(std::time::Duration::from_secs(2)));
}
