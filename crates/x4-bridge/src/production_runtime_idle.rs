use std::time::{Duration, Instant};

use x4_carrier_native::BridgePeer;

use crate::{OperationalHistory, ProductionLimits, ProductionObservationSession};

pub struct ReceiveProgress {
    origin: Instant,
    preparing: bool,
    preparation_limit: Duration,
    inactivity_limit: Duration,
}
impl ReceiveProgress {
    #[must_use]
    pub const fn issued(origin: Instant, limits: &ProductionLimits, heavy: bool) -> Self {
        let inactivity_limit = Duration::from_millis(limits.max_message_inactivity_millis as u64);
        Self {
            origin,
            preparing: heavy,
            preparation_limit: Duration::from_millis(limits.max_message_age_millis as u64),
            inactivity_limit,
        }
    }
    pub const fn received(&mut self, now: Instant) {
        self.origin = now;
        self.preparing = false;
    }
    fn expired(&self, now: Instant) -> bool {
        let limit = if self.preparing {
            self.preparation_limit
        } else {
            self.inactivity_limit
        };
        now.duration_since(self.origin) >= limit
    }
}

pub fn await_progress(
    peer: &mut BridgePeer,
    limits: &ProductionLimits,
    history: &mut OperationalHistory,
    session: &mut ProductionObservationSession,
    progress: &ReceiveProgress,
) -> Result<Vec<u8>, ()> {
    let idle_limit = Duration::from_millis(limits.max_message_inactivity_millis as u64);
    let poll_interval = idle_limit.min(Duration::from_millis(10));
    loop {
        match peer.receive_timeout(limits.complete_message_bytes, poll_interval) {
            Ok(Some(_)) if progress.expired(Instant::now()) => {
                let _ = history.record("rejected", "receive-timeout");
                return Err(());
            }
            Ok(Some(bytes)) => return Ok(bytes),
            Ok(None) if on_idle(history, session, progress) => return Err(()),
            Ok(None) => {}
            Err(_) => {
                let _ = history.record("rejected", "receive-failed");
                return Err(());
            }
        }
    }
}

fn on_idle(
    history: &mut OperationalHistory,
    session: &mut ProductionObservationSession,
    progress: &ReceiveProgress,
) -> bool {
    if session.expire_candidates(crate::production_runtime::now()) > 0 {
        let _ = history.record("rejected", "candidate-expired");
    }
    if !progress.expired(Instant::now()) {
        return false;
    }
    let _ = history.record("rejected", "receive-timeout");
    true
}

#[cfg(test)]
#[path = "production_runtime_idle_tests.rs"]
mod tests;
