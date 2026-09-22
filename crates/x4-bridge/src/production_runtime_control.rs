use crate::production_runtime::now;
use crate::production_runtime_idle::ReceiveProgress;
use crate::production_ship_schedule::ShipSchedule;
use crate::{OperationalHistory, ProductionLimits, ProductionObservationSession};
use observation_ingest::{
    CarrierControl, CarrierIdentity, CollectionIntentBody, ControlBody, DemandBody, HandshakeBody,
    encode_carrier_control,
};
use std::time::Instant;
use x4_carrier_native::BridgePeer;

pub struct RecoveryState<'a> {
    pub key: &'a mut String,
    pub progress: &'a mut ReceiveProgress,
    pub monotonic_millis: u64,
    pub issued_at: Instant,
}

pub fn initial(
    peer: &mut BridgePeer,
    identity: &CarrierIdentity,
    key: &str,
    revision: u64,
    limits: &ProductionLimits,
) -> Result<Instant, ()> {
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
    let issued = Instant::now();
    send(
        peer,
        identity,
        ControlBody::Demand(DemandBody { credit: 1 }),
        limits,
    )?;
    Ok(issued)
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

pub fn recover_idle(
    peer: &mut BridgePeer,
    identity: &CarrierIdentity,
    limits: &ProductionLimits,
    session: &mut ProductionObservationSession,
    schedule: &mut Option<ShipSchedule>,
    recovery: RecoveryState<'_>,
) -> bool {
    let RecoveryState {
        key,
        progress,
        monotonic_millis,
        issued_at,
    } = recovery;
    if observation_domain::ShipSectionIdentity::parse(key)
        .is_some_and(|section| section.kind() == observation_domain::ShipSectionKind::Core)
        || !session.stale_ship_parent(monotonic_millis)
    {
        return false;
    }
    let Some(active) = schedule else {
        return false;
    };
    active.complete();
    let Ok(next_key) = session.skip_stale_ship_scope() else {
        return false;
    };
    *key = next_key;
    let Some(issued) = issue(peer, identity, limits, session, schedule, key) else {
        return false;
    };
    *progress = ReceiveProgress::issued(issued_at.max(issued), limits, true);
    true
}

pub fn next(
    peer: &mut BridgePeer,
    identity: &CarrierIdentity,
    limits: &ProductionLimits,
    session: &mut ProductionObservationSession,
    schedule: &mut Option<ShipSchedule>,
    key: &mut String,
) -> Option<Instant> {
    // Reconcile completion before issuing fresh collection demand.
    if schedule.is_some() {
        *key = session.next_ship_key(key, now()).ok()?;
    }
    issue(peer, identity, limits, session, schedule, key)
}

fn issue(
    peer: &mut BridgePeer,
    identity: &CarrierIdentity,
    limits: &ProductionLimits,
    session: &mut ProductionObservationSession,
    schedule: &mut Option<ShipSchedule>,
    key: &str,
) -> Option<Instant> {
    if let Some(schedule) = schedule {
        if !schedule.admit(key) {
            return None;
        }
        let revision = session.heavy_revision_floor().ok()?;
        let issued_at = now();
        intent(peer, identity, key, revision, limits).ok()?;
        session.ship_intent_issued(identity, key, revision, issued_at);
    }
    let issued = Instant::now();
    send(
        peer,
        identity,
        ControlBody::Demand(DemandBody { credit: 1 }),
        limits,
    )
    .ok()?;
    Some(issued)
}

pub fn complete_selection(
    peer: &mut BridgePeer,
    identity: &CarrierIdentity,
    limits: &ProductionLimits,
    history: &mut OperationalHistory,
    session: &mut ProductionObservationSession,
    schedule: &mut Option<ShipSchedule>,
    key: &mut String,
) -> Option<Instant> {
    if session.maintain_heavy_history().is_err() {
        let _ = history.record("waiting", "heavy-retention-storage");
        return None;
    }
    if let Some(schedule) = schedule {
        schedule.complete();
    }
    let issued = next(peer, identity, limits, session, schedule, key);
    if issued.is_none() {
        let _ = history.record("waiting", "heavy-admission-or-peer-stop");
    }
    issued
}
