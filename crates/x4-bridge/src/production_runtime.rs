use crate::production_runtime_control::{initial, intent, send, wait_interval};
use crate::production_runtime_idle::await_progress;
use crate::production_runtime_message::disposition_name;
use crate::production_ship_schedule::ShipSchedule;
use crate::{OperationalHistory, PIPE_ENDPOINT, ProductionLimits, ProductionObservationSession};
use observation_ingest::{
    CarrierIdentity, ControlBody, DemandBody, ReceiverDisposition, decode_carrier_bootstrap,
};
use std::time::{Duration, Instant, SystemTime, UNIX_EPOCH};
use x4_carrier_native::{BridgePeer, TransportConfig};

pub fn run(
    limits: &ProductionLimits,
    history: &mut OperationalHistory,
    session: &mut ProductionObservationSession,
) -> ! {
    let config = TransportConfig {
        pipe_name: PIPE_ENDPOINT.into(),
        max_data_message_bytes: limits.complete_message_bytes,
        max_control_message_bytes: limits.control_message_bytes,
    };
    // One experiment window belongs to the bridge run, not every reconnect.
    let mut schedule = session.heavy_limits().and_then(ShipSchedule::new);
    loop {
        history.bind_session("", 0);
        let _ = history.record("waiting", "peer-absent");
        let scope = connect(&config, limits)
            .and_then(|mut peer| serve(&mut peer, limits, history, session, &mut schedule));
        if let Some(schedule) = &mut schedule {
            schedule.complete();
        }
        if let Some(scope) = scope {
            session.invalidate_source_scope(&scope);
        }
        std::thread::sleep(Duration::from_millis(
            limits.availability_interval_millis as u64,
        ));
    }
}
fn connect(config: &TransportConfig, limits: &ProductionLimits) -> Option<BridgePeer> {
    let timeout = Duration::from_millis(limits.reconnect_delay_millis as u64);
    (0..limits.reconnect_attempts).find_map(|_| BridgePeer::connect(config, timeout).ok())
}
fn serve(
    peer: &mut BridgePeer,
    limits: &ProductionLimits,
    history: &mut OperationalHistory,
    session: &mut ProductionObservationSession,
    schedule: &mut Option<ShipSchedule>,
) -> Option<observation_domain::SourceScopeId> {
    let bootstrap = peer
        .receive(limits.control_message_bytes)
        .map_err(|_| history.record("waiting", "bootstrap-read"))
        .ok()?;
    let identity = decode_carrier_bootstrap(&bootstrap, limits.control_message_bytes)
        .map_err(|_| history.record("waiting", "incompatible-bootstrap"))
        .ok()?;
    let revision = session
        .heavy_revision_floor()
        .map_err(|_| history.record("waiting", "revision-floor-storage"))
        .ok()?;
    history.bind_session(&identity.session_id, identity.epoch.get());
    if schedule
        .as_mut()
        .is_some_and(|value| !value.admit(session.collection_key()))
    {
        let _ = history.record("waiting", "heavy-admission-window");
        return None;
    }
    initial(peer, &identity, session.collection_key(), revision, limits).ok()?;
    let _ = history.record("collection", "compatible-session");
    receive_loop(peer, limits, history, session, &identity, schedule)
}
fn receive_loop(
    peer: &mut BridgePeer,
    limits: &ProductionLimits,
    history: &mut OperationalHistory,
    session: &mut ProductionObservationSession,
    identity: &CarrierIdentity,
    schedule: &mut Option<ShipSchedule>,
) -> Option<observation_domain::SourceScopeId> {
    let mut last_progress = Instant::now();
    let mut active_scope = None;
    let mut selected_key = session.collection_key().to_owned();
    while let Ok(bytes) = await_progress(peer, limits, history, session, last_progress) {
        last_progress = Instant::now();
        if let Some(schedule) = schedule {
            schedule.received(bytes.len());
        }
        history.bind_message("", "", 0);
        let (scope, disposition) = match crate::production_admission::admit(
            identity,
            &bytes,
            limits.complete_message_bytes,
            history,
            session,
            |body| send(peer, identity, body, limits),
        ) {
            Ok(value) => value,
            Err(error) => {
                return crate::production_admission::finish_error(&error, history, active_scope);
            }
        };
        active_scope = Some(scope);
        let state = if disposition == ReceiverDisposition::Committed {
            "committed"
        } else {
            "collection"
        };
        let _ = history.record(state, disposition_name(disposition));
        if disposition != ReceiverDisposition::Committed {
            continue;
        }
        if session.maintain_heavy_history().is_err() {
            let _ = history.record("waiting", "heavy-retention-storage");
            return active_scope;
        }
        if let Some(schedule) = schedule {
            schedule.complete();
        }
        if next(peer, identity, limits, session, schedule, &mut selected_key).is_none() {
            let _ = history.record("waiting", "heavy-admission-or-peer-stop");
            return active_scope;
        }
        last_progress = Instant::now();
    }
    let _ = history.record("waiting", "peer-disconnected");
    active_scope
}
fn next(
    peer: &mut BridgePeer,
    identity: &CarrierIdentity,
    limits: &ProductionLimits,
    session: &ProductionObservationSession,
    schedule: &mut Option<ShipSchedule>,
    key: &mut String,
) -> Option<()> {
    // Pump and reconcile the completion response before fresh source admission.
    wait_interval(peer, limits).ok()?;
    if let Some(schedule) = schedule {
        *key = session.next_ship_key(key).ok()?;
        if !schedule.admit(key) {
            return None;
        }
        intent(
            peer,
            identity,
            key,
            session.heavy_revision_floor().ok()?,
            limits,
        )
        .ok()?;
    }
    send(
        peer,
        identity,
        ControlBody::Demand(DemandBody { credit: 1 }),
        limits,
    )
    .ok()
}
pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |v| u64::try_from(v.as_millis()).unwrap_or(u64::MAX))
}
