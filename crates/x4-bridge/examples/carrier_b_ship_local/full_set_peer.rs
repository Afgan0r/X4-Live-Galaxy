use super::{Result, wire};
use observation_application::LifecycleResult;
use observation_domain::CompleteMessage;
use observation_ingest::{
    CarrierIdentity, CollectionIntentBody, ControlBody, DemandBody, HandshakeBody,
    ReceiverDisposition, decode_carrier_bootstrap, decode_complete_message,
};
use std::collections::BTreeMap;
use std::time::{Duration, Instant};
use x4_bridge::{HeavyShipLimits, PIPE_ENDPOINT, ProductionObservationSession};
use x4_carrier_native::{BridgePeer, TransportConfig};

pub struct RunSummary {
    pub incarnation: String,
    pub completions: BTreeMap<String, Vec<u64>>,
    pub blocked_skips: usize,
    pub closure_blocked: bool,
}

pub fn serve(
    receiver: &mut ProductionObservationSession,
    profile: &HeavyShipLimits,
    rotations: usize,
    block_scaleplate_once: bool,
) -> Result<RunSummary> {
    receiver.refresh_faction_census();
    let (mut peer, identity) = qualify(receiver, profile)?;
    let mut key = receiver.collection_key();
    let mut completions = BTreeMap::<String, Vec<u64>>::new();
    let mut cores = BTreeMap::<String, usize>::new();
    let mut blocked_skips = 0;
    loop {
        let revision = receiver
            .heavy_revision_floor()
            .map_err(|e| format!("floor:{e:?}"))?;
        receive_section(&mut peer, receiver, &identity, &key, revision, profile)?;
        completions.entry(key.clone()).or_default().push(revision);
        if let Some(faction) = key.strip_prefix("ship_core:") {
            *cores.entry(faction.to_owned()).or_default() += 1;
        }
        let mut next = if key == "faction_census" {
            receiver.collection_key()
        } else {
            receiver
                .next_ship_key(&key, wire::now())
                .map_err(|e| format!("next:{key}:{e:?}"))?
        };
        let mut recovered = false;
        if block_scaleplate_once && blocked_skips == 0 && next == "ship_cargo:scaleplate:g0" {
            recovered = super::full_set_recovery::recover(
                &mut peer, &identity, profile, receiver, &mut next,
            )?;
            blocked_skips = 1;
        }
        if next.starts_with("ship_core:")
            && ["argon", "scaleplate", "xenon", "khaak"]
                .iter()
                .all(|faction| cores.get(*faction).copied().unwrap_or(0) >= rotations)
        {
            break;
        }
        if !recovered {
            issue(&mut peer, &identity, &next, receiver, profile)?;
        }
        key = next;
    }
    Ok(RunSummary {
        incarnation: identity.producer_incarnation,
        completions,
        blocked_skips,
        closure_blocked: receiver.faction_census_blocks_closure(),
    })
}

fn qualify(
    receiver: &ProductionObservationSession,
    profile: &HeavyShipLimits,
) -> Result<(BridgePeer, CarrierIdentity)> {
    let config = TransportConfig {
        pipe_name: PIPE_ENDPOINT.into(),
        max_data_message_bytes: profile.bridge.complete_message_bytes,
        max_control_message_bytes: profile.bridge.control_message_bytes,
    };
    let deadline = Instant::now() + Duration::from_secs(10);
    let mut peer = loop {
        if let Ok(peer) = BridgePeer::connect(&config, Duration::from_millis(25)) {
            break peer;
        }
        if Instant::now() >= deadline {
            return Err("full-set pipe connect watchdog".into());
        }
        std::thread::sleep(Duration::from_millis(5));
    };
    let bootstrap = wire::receive(&mut peer)?;
    let identity = decode_carrier_bootstrap(&bootstrap, profile.bridge.control_message_bytes)
        .map_err(|e| format!("bootstrap:{e:?}"))?;
    wire::send(
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
    issue(
        &mut peer,
        &identity,
        &receiver.collection_key(),
        receiver,
        profile,
    )?;
    Ok((peer, identity))
}

fn issue(
    peer: &mut BridgePeer,
    identity: &CarrierIdentity,
    key: &str,
    receiver: &ProductionObservationSession,
    profile: &HeavyShipLimits,
) -> Result<()> {
    wire::send(
        peer,
        identity,
        ControlBody::CollectionIntent(CollectionIntentBody {
            section_key: key.into(),
            next_revision: receiver
                .heavy_revision_floor()
                .map_err(|e| format!("floor:{e:?}"))?,
            max_records: profile.bridge.max_candidate_records,
            max_raw_bytes: profile.bridge.max_candidate_raw_bytes,
            max_work: profile.bridge.max_candidate_work,
        }),
    )?;
    wire::send(
        peer,
        identity,
        ControlBody::Demand(DemandBody { credit: 1 }),
    )
}

fn receive_section(
    peer: &mut BridgePeer,
    receiver: &mut ProductionObservationSession,
    identity: &CarrierIdentity,
    key: &str,
    revision: u64,
    profile: &HeavyShipLimits,
) -> Result<()> {
    loop {
        let bytes = peer
            .receive_timeout(
                profile.bridge.complete_message_bytes,
                Duration::from_secs(10),
            )
            .map_err(|e| format!("full-set receive:{e:?}"))?
            .ok_or("full-set receive watchdog")?;
        let decoded = decode_complete_message(&bytes, profile.bridge.complete_message_bytes)
            .map_err(|e| format!("decode:{e:?}"))?;
        let (id, observed) = wire::message_identity(&bytes)?;
        let observed_key = match &decoded {
            CompleteMessage::SectionStart(value) => value.section_key.as_str(),
            CompleteMessage::ImmutableBatch(value) => value.section_key.as_str(),
            CompleteMessage::SectionCompletion(value) => value.section_key.as_str(),
            CompleteMessage::Control(_) => return Err("unexpected control".into()),
        };
        if observed != revision || observed_key != key {
            return Err(format!("full-set identity drift:{observed_key}:{observed}").into());
        }
        let done = matches!(decoded, CompleteMessage::SectionCompletion(_));
        let expected = if done {
            ReceiverDisposition::Committed
        } else {
            ReceiverDisposition::Received
        };
        let result = wire::submit(receiver, identity, &id, &bytes)
            .map_err(|e| format!("full-set submit:{key}:{revision}:{id}:{e}"))?;
        if result != LifecycleResult::Disposition(expected) {
            return Err(format!("full-set publication:{key}:{revision}:{result:?}").into());
        }
        wire::respond(
            peer,
            identity,
            &id,
            revision,
            &bytes,
            if done { "committed" } else { "received" },
        )?;
        if done {
            return Ok(());
        }
    }
}
