use std::fs::OpenOptions;
use std::io::{Read, Write};
use std::path::PathBuf;
use std::time::Duration;

use observation_ingest::{
    CarrierControl, CollectionIntentBody, CompleteMessage, ControlBody, DemandBody, HandshakeBody,
    ResetBody, decode_carrier_bootstrap, decode_complete_message, encode_carrier_control,
};
use x4_bridge::PIPE_ENDPOINT;
use x4_carrier_native::{BridgePeer, TransportConfig};

const CONTROL_BYTES: usize = 512;
const DATA_BYTES: usize = 1_048_576;
const RESOURCE_LIMIT: usize = 67_108_864;

fn main() -> Result<(), String> {
    let mut arguments = std::env::args_os().skip(1);
    let first = arguments
        .next()
        .ok_or_else(|| "missing output path".to_owned())?;
    if first == "--decode" {
        let input = arguments
            .next()
            .map(PathBuf::from)
            .ok_or_else(|| "missing input path".to_owned())?;
        return decode(&input);
    }
    capture(&PathBuf::from(first))
}

fn capture(output: &PathBuf) -> Result<(), String> {
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
    let received = peer
        .receive_timeout(DATA_BYTES, Duration::from_secs(30))
        .map_err(debug)?;
    let Some(bytes) = received else {
        return Err(timeout_error(reset(&mut peer, &identity)));
    };
    let decoded = decode_complete_message(&bytes, DATA_BYTES).map_err(debug);
    cleanup_result(decoded, reset(&mut peer, &identity))?;
    let mut file = OpenOptions::new()
        .write(true)
        .create_new(true)
        .open(output)
        .map_err(debug)?;
    file.write_all(&bytes).map_err(debug)?;
    file.sync_all().map_err(debug)?;
    Ok(())
}

fn reset(
    peer: &mut BridgePeer,
    identity: &observation_ingest::CarrierIdentity,
) -> Result<(), String> {
    send(
        peer,
        identity,
        ControlBody::Reset(ResetBody {
            reason: "diagnostic-capture-complete".to_owned(),
        }),
    )
}

fn decode(input: &PathBuf) -> Result<(), String> {
    let mut file = std::fs::File::open(input).map_err(debug)?;
    let bytes = read_bounded(&mut file)?;
    let kind = match decode_complete_message(&bytes, DATA_BYTES).map_err(debug)? {
        CompleteMessage::SectionStart(_) => "section_start",
        CompleteMessage::ImmutableBatch(_) => "immutable_batch",
        CompleteMessage::SectionCompletion(_) => "section_completion",
        CompleteMessage::Control(_) => "control",
    };
    writeln!(
        std::io::stdout().lock(),
        "decoded={kind} bytes={}",
        bytes.len()
    )
    .map_err(debug)?;
    Ok(())
}

fn read_bounded(input: &mut impl Read) -> Result<Vec<u8>, String> {
    let mut bytes = Vec::with_capacity(DATA_BYTES.min(64 * 1024));
    input
        .by_ref()
        .take((DATA_BYTES + 1) as u64)
        .read_to_end(&mut bytes)
        .map_err(debug)?;
    if bytes.len() > DATA_BYTES {
        return Err("input exceeds diagnostic decode limit".to_owned());
    }
    Ok(bytes)
}

fn timeout_error(reset: Result<(), String>) -> String {
    reset.map_or_else(
        |error| format!("pending message timed out; reset failed: {error}"),
        |()| "pending message timed out".to_owned(),
    )
}

fn cleanup_result<T>(primary: Result<T, String>, cleanup: Result<(), String>) -> Result<T, String> {
    match (primary, cleanup) {
        (Ok(value), Ok(())) => Ok(value),
        (Err(primary), Ok(())) => Err(primary),
        (Ok(_), Err(cleanup)) => Err(format!("reset failed: {cleanup}")),
        (Err(primary), Err(cleanup)) => Err(format!("{primary}; reset failed: {cleanup}")),
    }
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

#[cfg(test)]
mod tests {
    use super::{DATA_BYTES, cleanup_result, read_bounded, timeout_error};

    #[test]
    fn diagnostic_input_and_cleanup_failures_remain_bounded_and_visible() {
        let mut exact = std::io::Cursor::new(vec![0; DATA_BYTES]);
        assert_eq!(
            read_bounded(&mut exact).expect("exact input").len(),
            DATA_BYTES
        );
        let mut over = std::io::Cursor::new(vec![0; DATA_BYTES + 1]);
        assert!(read_bounded(&mut over).is_err());
        assert_eq!(timeout_error(Ok(())), "pending message timed out");
        assert_eq!(
            timeout_error(Err("closed".to_owned())),
            "pending message timed out; reset failed: closed"
        );
        assert_eq!(
            cleanup_result::<()>(Err("decode failed".to_owned()), Err("closed".to_owned())),
            Err("decode failed; reset failed: closed".to_owned())
        );
    }
}
