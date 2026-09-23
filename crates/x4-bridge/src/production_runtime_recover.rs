use crate::production_runtime_control::recover_idle_selection;
use crate::production_runtime_idle::{ProgressError, await_progress};
use crate::{
    OperationalHistory, ProductionLimits, ProductionObservationSession, RecoveryState, ShipSchedule,
};
use observation_ingest::CarrierIdentity;
use x4_carrier_native::BridgePeer;

pub fn state<'a>(
    key: &'a mut String,
    progress: &'a mut crate::ReceiveProgress,
) -> RecoveryState<'a> {
    RecoveryState {
        key,
        progress,
        monotonic_millis: crate::production_runtime::now(),
        issued_at: std::time::Instant::now(),
    }
}

pub fn receive(
    peer: &mut BridgePeer,
    identity: &CarrierIdentity,
    limits: &ProductionLimits,
    history: &mut OperationalHistory,
    session: &mut ProductionObservationSession,
    schedule: &mut Option<ShipSchedule>,
    mut recovery: RecoveryState<'_>,
) -> Result<Option<Vec<u8>>, ProgressError> {
    match await_progress(peer, limits, history, session, recovery.progress) {
        Ok(bytes) => Ok(Some(bytes)),
        Err(ProgressError::Timeout) => {
            recovery.monotonic_millis = crate::production_runtime::now();
            let Some(issued) =
                recover_idle_selection(peer, identity, limits, session, schedule, recovery)
            else {
                return Err(ProgressError::Timeout);
            };
            history.bind_selection(issued.attempt, &issued.key, issued.revision);
            let _ = history.record("recovered", "stale-scope-rotated");
            Ok(None)
        }
        Err(error) => Err(error),
    }
}
