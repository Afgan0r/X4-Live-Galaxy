use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::time::Duration;

use observation_ingest::{
    CarrierControl, CollectionIntentBody, ControlBody, DemandBody, HandshakeBody,
    decode_carrier_bootstrap, encode_carrier_control,
};
use x4_bridge::PIPE_ENDPOINT;
use x4_carrier_native::{BridgePeer, TransportConfig};

const CONTROL_BYTES: usize = 512;
const DATA_BYTES: usize = 1_048_576;
const RESOURCE_LIMIT: usize = 67_108_864;

fn main() -> Result<(), String> {
    let output = std::env::args_os()
        .nth(1)
        .map(PathBuf::from)
        .ok_or_else(|| "missing output path".to_owned())?;
    let config = TransportConfig {
        pipe_name: PIPE_ENDPOINT.into(),
        max_data_message_bytes: DATA_BYTES,
        max_control_message_bytes: CONTROL_BYTES,
    };
    let mut peer = BridgePeer::connect(&config, Duration::from_secs(30)).map_err(debug)?;
    let bootstrap = peer.receive(CONTROL_BYTES).map_err(debug)?;
    let identity = decode_carrier_bootstrap(&bootstrap, CONTROL_BYTES).map_err(debug)?;
    send(
        &mut peer,
        &identity,
        ControlBody::Handshake(HandshakeBody {
            native_abi: 2,
            envelope_contract: 2,
            schema_version: 1,
            policy_version: 2,
            canonicalization_version: 3,
            digest_version: 1,
        }),
    )?;
    send(
        &mut peer,
        &identity,
        ControlBody::CollectionIntent(CollectionIntentBody {
            section_key: "ship_core".into(),
            next_revision: 1,
            max_records: RESOURCE_LIMIT,
            max_raw_bytes: RESOURCE_LIMIT,
            max_work: RESOURCE_LIMIT,
        }),
    )?;
    send(
        &mut peer,
        &identity,
        ControlBody::Demand(DemandBody { credit: 1 }),
    )?;
    let bytes = peer
        .receive_timeout(DATA_BYTES, Duration::from_secs(30))
        .map_err(debug)?
        .ok_or_else(|| "pending message timed out".to_owned())?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(debug)?;
    file.write_all(&bytes).map_err(debug)?;
    file.sync_all().map_err(debug)?;
    Ok(())
}

fn send(
    peer: &mut BridgePeer,
    identity: &observation_ingest::CarrierIdentity,
    body: ControlBody,
) -> Result<(), String> {
    let bytes = encode_carrier_control(
        &CarrierControl {
            identity: identity.clone(),
            body,
        },
        CONTROL_BYTES,
    )
    .map_err(debug)?;
    peer.send_control(&bytes).map_err(debug)?;
    Ok(())
}

fn debug(error: impl std::fmt::Debug) -> String {
    format!("{error:?}")
}
