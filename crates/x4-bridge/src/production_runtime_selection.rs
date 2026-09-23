use super::failure::SelectionFailure;
use super::{intent, send};
use crate::production_runtime::now;
use crate::production_ship_schedule::ShipSchedule;
use crate::{OperationalHistory, ProductionLimits, ProductionObservationSession};
use observation_ingest::{CarrierIdentity, ControlBody, DemandBody};
use std::time::Instant;
use x4_carrier_native::BridgePeer;

pub struct IssuedSelection {
    pub at: Instant,
    pub revision: Option<u64>,
    pub attempt: u64,
    pub key: String,
}

type SelectionError = (SelectionFailure, Option<u64>);

fn next(
    peer: &mut BridgePeer,
    identity: &CarrierIdentity,
    limits: &ProductionLimits,
    session: &mut ProductionObservationSession,
    schedule: &mut Option<ShipSchedule>,
    key: &mut String,
    attempt: u64,
) -> Result<IssuedSelection, SelectionError> {
    if schedule.is_some() {
        *key = session
            .next_ship_key(key, now())
            .map_err(|_| (SelectionFailure::Cursor, None))?;
    }
    issue(peer, identity, limits, session, schedule, key, attempt)
}

pub(super) fn issue(
    peer: &mut BridgePeer,
    identity: &CarrierIdentity,
    limits: &ProductionLimits,
    session: &mut ProductionObservationSession,
    schedule: &mut Option<ShipSchedule>,
    key: &str,
    attempt: u64,
) -> Result<IssuedSelection, SelectionError> {
    let mut revision = None;
    if let Some(schedule) = schedule {
        if !schedule.admit(key) {
            return Err((SelectionFailure::Admission, None));
        }
        let next_revision = session
            .heavy_revision_floor()
            .map_err(|_| (SelectionFailure::Revision, None))?;
        revision = Some(next_revision);
        let issued_at = now();
        intent(peer, identity, key, next_revision, limits)
            .map_err(|()| (SelectionFailure::Intent, revision))?;
        session.ship_intent_issued(identity, key, next_revision, issued_at);
    }
    let issued = Instant::now();
    send(
        peer,
        identity,
        ControlBody::Demand(DemandBody { credit: 1 }),
        limits,
    )
    .map_err(|()| (SelectionFailure::Demand, revision))?;
    Ok(IssuedSelection {
        at: issued,
        revision,
        attempt,
        key: key.to_owned(),
    })
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
    let attempt = schedule.as_mut().map_or(0, ShipSchedule::next_attempt);
    match next(peer, identity, limits, session, schedule, key, attempt) {
        Ok(issued) => {
            history.bind_selection(issued.attempt, &issued.key, issued.revision);
            Some(issued.at)
        }
        Err((error, revision)) => {
            let section = if matches!(error, SelectionFailure::Cursor) {
                ""
            } else {
                key.as_str()
            };
            history.bind_selection(attempt, section, revision);
            let _ = history.record("rejected", error.reason());
            None
        }
    }
}
