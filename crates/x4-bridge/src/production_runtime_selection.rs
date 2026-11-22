use super::failure::SelectionFailure;
use super::{intent, send};
use crate::production_runtime::now;
use crate::production_ship_schedule::ShipSchedule;
use crate::{OperationalHistory, ProductionError, ProductionLimits, ProductionObservationSession};
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

#[derive(Clone, Copy)]
pub(super) struct SelectionDemand<'a> {
    key: &'a str,
    revision_floor: Option<Result<u64, ProductionError>>,
}

impl<'a> SelectionDemand<'a> {
    pub(super) const fn new(key: &'a str) -> Self {
        Self {
            key,
            revision_floor: None,
        }
    }
}

fn next(
    peer: &mut BridgePeer,
    identity: &CarrierIdentity,
    limits: &ProductionLimits,
    session: &mut ProductionObservationSession,
    schedule: &mut Option<ShipSchedule>,
    key: &mut String,
    attempt: u64,
) -> Result<IssuedSelection, SelectionError> {
    let revision_floor = if schedule.is_some() {
        let selection = session
            .next_ship_selection(key, now())
            .map_err(|_| (SelectionFailure::Cursor, None))?;
        *key = selection.key;
        selection.revision_floor
    } else {
        None
    };
    let demand = SelectionDemand {
        key,
        revision_floor,
    };
    issue(peer, identity, limits, session, schedule, demand, attempt)
}

pub(super) fn issue(
    peer: &mut BridgePeer,
    identity: &CarrierIdentity,
    limits: &ProductionLimits,
    session: &mut ProductionObservationSession,
    schedule: &mut Option<ShipSchedule>,
    demand: SelectionDemand<'_>,
    attempt: u64,
) -> Result<IssuedSelection, SelectionError> {
    let SelectionDemand {
        key,
        revision_floor,
    } = demand;
    let mut revision = None;
    if let Some(schedule) = schedule {
        schedule
            .try_admit(key)
            .map_err(|denial| (SelectionFailure::Admission(denial), None))?;
        let next_revision = revision_floor
            .unwrap_or_else(|| session.heavy_revision_floor())
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
            error.record(history, attempt, key, revision);
            None
        }
    }
}
