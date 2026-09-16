use super::{Result, wire};
use observation_application::LifecycleResult;
use observation_ingest::{
    CarrierIdentity, CollectionIntentBody, ControlBody, DemandBody, ReceiverDisposition,
};
use x4_bridge::ProductionObservationSession;
use x4_carrier_native::BridgePeer;
type Completion = (CarrierIdentity, Vec<u8>);
type Trace = (String, Vec<Completion>);

pub fn serve(receiver: &mut ProductionObservationSession) -> Result<Trace> {
    let (mut peer, identity, _) = super::peer::qualify(receiver)?;
    let mut completions = Vec::new();
    for (index, (key, revision, count)) in [
        ("ship_core", 1, 8),
        ("ship_cargo:g0", 2, 4),
        ("ship_cargo:g1", 3, 4),
        ("ship_cargo:g0", 4, 4),
        ("ship_cargo:g1", 5, 4),
    ]
    .into_iter()
    .enumerate()
    {
        if index > 0 {
            demand(&mut peer, &identity, key, revision)?;
        }
        let bytes = cycle(&mut peer, receiver, &identity, key, revision, count)?;
        completions.push((identity.clone(), bytes));
    }
    Ok((identity.producer_incarnation.clone(), completions))
}

fn demand(
    peer: &mut BridgePeer,
    identity: &CarrierIdentity,
    key: &str,
    revision: u64,
) -> Result<()> {
    if peer
        .receive_timeout(4096, std::time::Duration::from_secs(5))
        .map_err(|e| format!("interval:{e:?}"))?
        .is_some()
    {
        return Err("message before demand".into());
    }
    wire::send(
        peer,
        identity,
        ControlBody::CollectionIntent(CollectionIntentBody {
            section_key: key.into(),
            next_revision: revision,
            max_records: 16,
            max_raw_bytes: 512,
            max_work: 129,
        }),
    )?;
    wire::send(
        peer,
        identity,
        ControlBody::Demand(DemandBody { credit: 1 }),
    )
}

fn cycle(
    peer: &mut BridgePeer,
    receiver: &mut ProductionObservationSession,
    identity: &CarrierIdentity,
    key: &str,
    revision: u64,
    count: usize,
) -> Result<Vec<u8>> {
    let mut last = Vec::new();
    for ordinal in 0..count + 2 {
        let bytes = wire::receive(peer)?;
        let (id, observed) = wire::message_identity(&bytes)?;
        if observed != revision {
            return Err("detail revision drift".into());
        }
        let done = ordinal == count + 1;
        let expected = if done {
            ReceiverDisposition::Committed
        } else {
            ReceiverDisposition::Received
        };
        let result = wire::submit(receiver, identity, &id, &bytes)?;
        if result != LifecycleResult::Disposition(expected) {
            return Err(format!("detail publication key={key} revision={revision} ordinal={ordinal} result={result:?}").into());
        }
        wire::respond(
            peer,
            identity,
            &id,
            revision,
            &bytes,
            if done { "committed" } else { "received" },
        )?;
        last = bytes;
    }
    Ok(last)
}
