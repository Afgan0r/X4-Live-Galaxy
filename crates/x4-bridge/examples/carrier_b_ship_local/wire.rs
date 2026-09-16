use super::Result;
use observation_application::LifecycleResult;
use observation_domain::{BatchId, CompleteMessage};
use observation_ingest::{
    CarrierControl, CarrierIdentity, ControlBody, DispositionBody, complete_message_digest,
    decode_complete_message, encode_carrier_control,
};
use std::fmt::Write as _;
use std::time::Duration;
use x4_bridge::ProductionObservationSession;
use x4_carrier_native::BridgePeer;
pub(super) fn submit(
    receiver: &mut ProductionObservationSession,
    identity: &CarrierIdentity,
    id: &str,
    bytes: &[u8],
) -> Result<LifecycleResult> {
    receiver
        .submit_received(
            identity.epoch,
            BatchId::new(id).ok_or("id")?,
            bytes.to_vec(),
            1,
            now(),
        )
        .map_err(|e| format!("receiver:{e:?}").into())
}
pub(super) fn receive(peer: &mut BridgePeer) -> Result<Vec<u8>> {
    peer.receive_timeout(4096, Duration::from_secs(10))
        .map_err(|e| format!("pipe read:{e:?}"))?
        .ok_or_else(|| "pipe read watchdog".into())
}
pub(super) fn send(
    peer: &mut BridgePeer,
    identity: &CarrierIdentity,
    body: ControlBody,
) -> Result<()> {
    let bytes = encode_carrier_control(
        &CarrierControl {
            identity: identity.clone(),
            body,
        },
        512,
    )
    .map_err(|e| format!("control encode:{e:?}"))?;
    peer.send_control(&bytes)
        .map_err(|e| format!("control write:{e:?}").into())
}
pub(super) fn respond(
    peer: &mut BridgePeer,
    identity: &CarrierIdentity,
    id: &str,
    revision: u64,
    bytes: &[u8],
    result: &str,
) -> Result<()> {
    let mut digest = String::new();
    for byte in complete_message_digest(bytes) {
        write!(&mut digest, "{byte:02x}")?;
    }
    send(
        peer,
        identity,
        ControlBody::Disposition(DispositionBody {
            message_id: id.into(),
            section_key: "ship_core".into(),
            section_revision: revision,
            message_digest: digest,
            disposition: result.into(),
        }),
    )
}
pub(super) fn message_identity(bytes: &[u8]) -> Result<(String, u64)> {
    match decode_complete_message(bytes, 4096).map_err(|e| format!("decode:{e:?}"))? {
        CompleteMessage::SectionStart(v) => Ok((
            format!("message:start:ship_core:{}", v.section_revision.get()),
            v.section_revision.get(),
        )),
        CompleteMessage::ImmutableBatch(v) => {
            Ok((v.batch_id.as_str().into(), v.section_revision.get()))
        }
        CompleteMessage::SectionCompletion(v) => Ok((
            format!("message:complete:ship_core:{}", v.section_revision.get()),
            v.section_revision.get(),
        )),
        CompleteMessage::Control(_) => Err("unexpected control".into()),
    }
}
pub(super) fn now() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map_or(0, |v| u64::try_from(v.as_millis()).unwrap_or(u64::MAX))
}
