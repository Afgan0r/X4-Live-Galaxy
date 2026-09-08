use std::time::{Duration, Instant};

use x4_carrier_native::BridgePeer;

use crate::{OperationalHistory, ProductionLimits, ProductionObservationSession};

pub fn await_progress(
    peer: &mut BridgePeer,
    limits: &ProductionLimits,
    history: &mut OperationalHistory,
    session: &mut ProductionObservationSession,
    last_progress: Instant,
) -> Result<Vec<u8>, ()> {
    let idle_limit = Duration::from_millis(limits.max_message_inactivity_millis as u64);
    let poll_interval = idle_limit.min(Duration::from_millis(10));
    loop {
        match peer.receive_timeout(limits.complete_message_bytes, poll_interval) {
            Ok(Some(bytes)) => return Ok(bytes),
            Ok(None) if on_idle(history, session, last_progress, idle_limit) => return Err(()),
            Ok(None) => {}
            Err(_) => return Err(()),
        }
    }
}

fn on_idle(
    history: &mut OperationalHistory,
    session: &mut ProductionObservationSession,
    last_progress: Instant,
    idle_limit: Duration,
) -> bool {
    if session.expire_candidates(crate::production_runtime::now()) > 0 {
        let _ = history.record("rejected", "candidate-expired");
    }
    if last_progress.elapsed() < idle_limit {
        return false;
    }
    let _ = history.record("disconnected", "peer-inactive");
    true
}
