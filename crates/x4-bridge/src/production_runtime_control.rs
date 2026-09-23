use crate::production_runtime_idle::ReceiveProgress;
use crate::production_ship_schedule::ShipSchedule;
use crate::{ProductionLimits, ProductionObservationSession};
use observation_ingest::{
    CarrierControl, CarrierIdentity, CollectionIntentBody, ControlBody, DemandBody, HandshakeBody,
    encode_carrier_control,
};
use std::time::Instant;
use x4_carrier_native::BridgePeer;
#[path = "production_control_failure.rs"]
mod failure;
#[path = "production_runtime_selection.rs"]
mod selection;
pub use selection::IssuedSelection;
pub use selection::complete_selection;

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
    recover_idle_selection(peer, identity, limits, session, schedule, recovery).is_some()
}

pub fn recover_idle_selection(
    peer: &mut BridgePeer,
    identity: &CarrierIdentity,
    limits: &ProductionLimits,
    session: &mut ProductionObservationSession,
    schedule: &mut Option<ShipSchedule>,
    recovery: RecoveryState<'_>,
) -> Option<IssuedSelection> {
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
        return None;
    }
    let Some(active) = schedule else {
        return None;
    };
    active.complete();
    let next_key = session.skip_stale_ship_scope().ok()?;
    *key = next_key;
    let attempt = schedule.as_mut()?.next_attempt();
    let issued = selection::issue(peer, identity, limits, session, schedule, key, attempt).ok()?;
    *progress = ReceiveProgress::issued(issued_at.max(issued.at), limits, true);
    Some(issued)
}
