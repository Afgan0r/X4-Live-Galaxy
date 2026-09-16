type Trace = (String, (CarrierIdentity, Vec<u8>));
use super::Result;
use observation_application::LifecycleResult;
use observation_domain::{BatchId, SectionKey};
use observation_ingest::{
    CarrierIdentity, CollectionIntentBody, ControlBody, DemandBody, HandshakeBody,
    ReceiverDisposition, decode_carrier_bootstrap,
};
#[path = "wire.rs"]
mod wire;
use std::time::{Duration, Instant};
use wire::{message_identity, now, receive, respond, send, submit};
use x4_bridge::{PIPE_ENDPOINT, ProductionObservationSession};
use x4_carrier_native::{BridgePeer, TransportConfig};

pub fn serve(
    receiver: &mut ProductionObservationSession,
    commits: usize,
    malformed: bool,
) -> Result<Trace> {
    let (mut peer, identity, floor) = qualify(receiver)?;
    let mut last = Vec::new();
    for cycle in 0..commits {
        let revision = floor + cycle as u64;
        last = receive_cycle(&mut peer, receiver, &identity, revision)?;
        if cycle + 1 < commits || malformed {
            next_demand(&mut peer, &identity)?;
        }
    }
    if malformed {
        reject_replacement(&mut peer, receiver, &identity, floor + commits as u64)?;
    }
    Ok((identity.producer_incarnation.clone(), (identity, last)))
}

pub fn replay(
    receiver: &mut ProductionObservationSession,
    completion: &(CarrierIdentity, Vec<u8>),
) -> Result<()> {
    let (id, _) = message_identity(&completion.1)?;
    if submit(receiver, &completion.0, &id, &completion.1)?
        != LifecycleResult::Disposition(ReceiverDisposition::Committed)
    {
        return Err("exact completion replay".into());
    }
    Ok(())
}

fn reject_replacement(
    peer: &mut BridgePeer,
    receiver: &mut ProductionObservationSession,
    identity: &CarrierIdentity,
    expected_revision: u64,
) -> Result<()> {
    let start = receive(peer)?;
    let (id, revision) = message_identity(&start)?;
    if revision != expected_revision
        || submit(receiver, identity, &id, &start)?
            != LifecycleResult::Disposition(ReceiverDisposition::Received)
    {
        return Err("malformed replacement start".into());
    }
    respond(peer, identity, &id, revision, &start, "received")?;
    let bytes = receive(peer)?;
    let (id, observed) = message_identity(&bytes)?;
    if observed != revision {
        return Err("malformed revision identity".into());
    }
    let rejected = receiver.submit_received(
        identity.epoch,
        BatchId::new(id.clone()).ok_or("id")?,
        bytes.clone(),
        1,
        now(),
    );
    if !matches!(
        rejected,
        Err(x4_bridge::ProductionError::Lifecycle(
            observation_application::LifecycleError::AuthorityRejected
        ))
    ) {
        return Err(format!("malformed receiver disposition:{rejected:?}").into());
    }
    respond(
        peer,
        identity,
        &id,
        revision,
        &bytes,
        "permanently_rejected",
    )?;
    Ok(())
}

fn qualify(receiver: &ProductionObservationSession) -> Result<(BridgePeer, CarrierIdentity, u64)> {
    let config = TransportConfig {
        pipe_name: PIPE_ENDPOINT.into(),
        max_data_message_bytes: 4096,
        max_control_message_bytes: 512,
    };
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut peer = loop {
        if let Ok(peer) = BridgePeer::connect(&config, Duration::from_millis(25)) {
            break peer;
        }
        if Instant::now() >= deadline {
            return Err("pipe connect watchdog".into());
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    let bootstrap = receive(&mut peer)?;
    let identity =
        decode_carrier_bootstrap(&bootstrap, 512).map_err(|e| format!("bootstrap:{e:?}"))?;
    let key = SectionKey::new("ship_core").ok_or("key")?;
    let floor = receiver
        .next_revision(&key)
        .map_err(|e| format!("floor:{e:?}"))?;
    for body in [
        ControlBody::Handshake(HandshakeBody {
            native_abi: 2,
            envelope_contract: 2,
            schema_version: 1,
            policy_version: 2,
            canonicalization_version: 3,
            digest_version: 1,
        }),
        ControlBody::CollectionIntent(CollectionIntentBody {
            section_key: "ship_core".into(),
            next_revision: floor,
            max_records: 16,
            max_raw_bytes: 512,
            max_work: 129,
        }),
        ControlBody::Demand(DemandBody { credit: 1 }),
    ] {
        send(&mut peer, &identity, body)?;
    }
    Ok((peer, identity, floor))
}

fn receive_cycle(
    peer: &mut BridgePeer,
    receiver: &mut ProductionObservationSession,
    identity: &CarrierIdentity,
    revision: u64,
) -> Result<Vec<u8>> {
    let mut last = Vec::new();
    for ordinal in 0..10 {
        let bytes = receive(peer)?;
        let (id, observed) = message_identity(&bytes)?;
        if observed != revision {
            return Err("revision identity drift".into());
        }
        let result = submit(receiver, identity, &id, &bytes)?;
        let expected = if ordinal == 9 {
            ReceiverDisposition::Committed
        } else {
            ReceiverDisposition::Received
        };
        if result != LifecycleResult::Disposition(expected) {
            return Err(format!("stage {ordinal}:{result:?}").into());
        }
        respond(
            peer,
            identity,
            &id,
            revision,
            &bytes,
            if ordinal == 9 {
                "committed"
            } else {
                "received"
            },
        )?;
        if ordinal == 9 {
            last = bytes;
        }
    }
    Ok(last)
}

fn next_demand(peer: &mut BridgePeer, identity: &CarrierIdentity) -> Result<()> {
    // Cross the producer's approved interval on this same pipe; no inactivity reconnect.
    if peer
        .receive_timeout(4096, Duration::from_secs(5))
        .map_err(|e| format!("demand interval:{e:?}"))?
        .is_some()
    {
        return Err("message before fresh demand".into());
    }
    send(
        peer,
        identity,
        ControlBody::Demand(DemandBody { credit: 1 }),
    )?;
    Ok(())
}
