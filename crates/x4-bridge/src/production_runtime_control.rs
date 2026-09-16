use crate::ProductionLimits;
use observation_ingest::{
    CarrierControl, CarrierIdentity, CollectionIntentBody, ControlBody, DemandBody, HandshakeBody,
    encode_carrier_control,
};
use std::time::Duration;
use x4_carrier_native::BridgePeer;

pub fn wait_interval(
    peer: &mut BridgePeer,
    limits: &ProductionLimits,
    interval: usize,
) -> Result<(), ()> {
    match peer.receive_timeout(
        limits.complete_message_bytes,
        Duration::from_millis(interval as u64),
    ) {
        Ok(None) => Ok(()),
        _ => Err(()),
    }
}
pub fn initial(
    peer: &mut BridgePeer,
    identity: &CarrierIdentity,
    key: &str,
    revision: u64,
    limits: &ProductionLimits,
) -> Result<(), ()> {
    send(
        peer,
        identity,
        ControlBody::Handshake(HandshakeBody {
            native_abi: 2,
            envelope_contract: 2,
            schema_version: 1,
            policy_version: 2,
            canonicalization_version: 3,
            digest_version: 1,
        }),
        limits,
    )?;
    intent(peer, identity, key, revision, limits)?;
    send(
        peer,
        identity,
        ControlBody::Demand(DemandBody { credit: 1 }),
        limits,
    )
}
pub fn intent(
    peer: &mut BridgePeer,
    identity: &CarrierIdentity,
    key: &str,
    revision: u64,
    limits: &ProductionLimits,
) -> Result<(), ()> {
    send(
        peer,
        identity,
        ControlBody::CollectionIntent(CollectionIntentBody {
            section_key: key.into(),
            next_revision: revision,
            max_records: limits.max_candidate_records,
            max_raw_bytes: limits.max_candidate_raw_bytes,
            max_work: limits.max_candidate_work,
        }),
        limits,
    )
}
pub fn send(
    peer: &mut BridgePeer,
    identity: &CarrierIdentity,
    body: ControlBody,
    limits: &ProductionLimits,
) -> Result<(), ()> {
    let bytes = encode_carrier_control(
        &CarrierControl {
            identity: identity.clone(),
            body,
        },
        limits.control_message_bytes,
    )
    .map_err(|_| ())?;
    peer.send_control(&bytes).map_err(|_| ())
}
