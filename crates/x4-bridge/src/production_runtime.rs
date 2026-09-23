use crate::production_runtime_control::{complete_selection, initial, send};
use crate::production_runtime_idle::{ProgressError, ReceiveProgress};
use crate::production_runtime_message::{record_disposition, selection_finished};
use crate::production_ship_schedule::ShipSchedule;
use crate::{OperationalHistory, PIPE_ENDPOINT, ProductionLimits, ProductionObservationSession};
use observation_ingest::{CarrierIdentity, decode_carrier_bootstrap};
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
        let scope = connect(&config, limits, history)
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
fn connect(
    config: &TransportConfig,
    limits: &ProductionLimits,
    history: &mut OperationalHistory,
) -> Option<BridgePeer> {
    let timeout = Duration::from_millis(limits.reconnect_delay_millis as u64);
    (1..=limits.reconnect_attempts).find_map(|attempt| {
        BridgePeer::connect(config, timeout).map_or_else(
            |_| {
                history.bind_message(&format!("attempt-{attempt}"), "transport", 0);
                let _ = history.record("waiting", "connect-failed");
                None
            },
            Some,
        )
    })
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
    let collection_key = session.collection_key();
    if schedule
        .as_mut()
        .is_some_and(|value| !value.admit(&collection_key))
    {
        let _ = history.record("waiting", "heavy-admission-window");
        return None;
    }
    let issued = initial(peer, &identity, &collection_key, revision, limits).map_or_else(
        |()| {
            history.bind_message("attempt-1", &collection_key, revision);
            let _ = history.record("rejected", "control-send-failed");
            None
        },
        Some,
    )?;
    let _ = history.record("collection", "compatible-session");
    receive_loop(peer, limits, history, session, &identity, schedule, issued)
}
fn receive_loop(
    peer: &mut BridgePeer,
    limits: &ProductionLimits,
    history: &mut OperationalHistory,
    session: &mut ProductionObservationSession,
    identity: &CarrierIdentity,
    schedule: &mut Option<ShipSchedule>,
    issued: Instant,
) -> Option<observation_domain::SourceScopeId> {
    let mut progress = ReceiveProgress::issued(issued, limits, schedule.is_some());
    let mut active_scope = None;
    let mut selected_key = session.collection_key();
    let exit = loop {
        let bytes = match crate::production_runtime_recover::receive(
            peer,
            identity,
            limits,
            history,
            session,
            schedule,
            crate::production_runtime_recover::state(&mut selected_key, &mut progress),
        ) {
            Ok(Some(bytes)) => bytes,
            Ok(None) => continue,
            Err(ProgressError::Timeout) => {
                let _ = history.record("rejected", "receive-timeout");
                break ProgressError::Timeout;
            }
            Err(ProgressError::Receive) => break ProgressError::Receive,
        };
        progress.received(Instant::now());
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
        record_disposition(history, disposition);
        if !selection_finished(disposition, schedule.is_some()) {
            continue;
        }
        let Some(issued) = complete_selection(
            peer,
            identity,
            limits,
            history,
            session,
            schedule,
            &mut selected_key,
        ) else {
            return active_scope;
        };
        progress = ReceiveProgress::issued(issued, limits, schedule.is_some());
    };
    let reason = match exit {
        ProgressError::Timeout => "receive-timeout-exhausted",
        ProgressError::Receive => "peer-disconnected",
    };
    let _ = history.record("waiting", reason);
    active_scope
}

pub fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |v| u64::try_from(v.as_millis()).unwrap_or(u64::MAX))
}
