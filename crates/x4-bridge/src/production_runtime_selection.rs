use super::failure::SelectionFailure;
use super::{intent, send};
use crate::production_runtime::now;
use crate::production_ship_schedule::ShipSchedule;
use crate::{OperationalHistory, ProductionLimits, ProductionObservationSession};
use observation_ingest::{CarrierIdentity, ControlBody, DemandBody};
use std::time::Instant;
use x4_carrier_native::BridgePeer;

fn next(
    peer: &mut BridgePeer,
    identity: &CarrierIdentity,
    limits: &ProductionLimits,
    session: &mut ProductionObservationSession,
    schedule: &mut Option<ShipSchedule>,
    key: &mut String,
) -> Result<Instant, SelectionFailure> {
    if schedule.is_some() {
        *key = session
            .next_ship_key(key, now())
            .map_err(|_| SelectionFailure::Cursor)?;
    }
    issue(peer, identity, limits, session, schedule, key)
}

pub(super) fn issue(
    peer: &mut BridgePeer,
    identity: &CarrierIdentity,
    limits: &ProductionLimits,
    session: &mut ProductionObservationSession,
    schedule: &mut Option<ShipSchedule>,
    key: &str,
) -> Result<Instant, SelectionFailure> {
    if let Some(schedule) = schedule {
        if !schedule.admit(key) {
            return Err(SelectionFailure::Admission);
        }
        let revision = session
            .heavy_revision_floor()
            .map_err(|_| SelectionFailure::Revision)?;
        let issued_at = now();
        intent(peer, identity, key, revision, limits).map_err(|()| SelectionFailure::Intent)?;
        session.ship_intent_issued(identity, key, revision, issued_at);
    }
    let issued = Instant::now();
    send(
        peer,
        identity,
        ControlBody::Demand(DemandBody { credit: 1 }),
        limits,
    )
    .map_err(|()| SelectionFailure::Demand)?;
    Ok(issued)
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
    if session.heavy_collection_complete() {
        let _ = history.record("completed", "no-included-factions");
        return None;
    }
    match next(peer, identity, limits, session, schedule, key) {
        Ok(issued) => {
            let revision = session.heavy_revision_floor().unwrap_or(0);
            history.bind_message("attempt-1", key, revision);
            Some(issued)
        }
        Err(error) => {
            let revision = session.heavy_revision_floor().unwrap_or(0);
            history.bind_message("attempt-1", key, revision);
            let _ = history.record("rejected", error.reason());
            None
        }
    }
}
