use crate::HeavyShipLimits;
use observation_ingest::{
    CollectionClass, CollectionIntent, CollectionIntentId, CollectionPolicyLimits, DeliveredPulse,
    MonotonicClock, ObservationScheduler, SchedulerSafetyLimits, TransportPolicyLimits, WorkKind,
};
use std::time::Instant;

struct RuntimeClock(Instant);
impl MonotonicClock for RuntimeClock {
    fn now_millis(&self) -> u64 {
        u64::try_from(self.0.elapsed().as_millis()).unwrap_or(u64::MAX)
    }
}

pub struct ShipSchedule {
    started: Instant,
    window: u128,
    declared: usize,
    available: usize,
    scheduler: ObservationScheduler<RuntimeClock>,
    active: Option<CollectionIntentId>,
    received_work: usize,
}
impl ShipSchedule {
    pub fn new(profile: &HeavyShipLimits) -> Option<Self> {
        let started = Instant::now();
        Some(Self {
            started,
            window: u128::try_from(profile.admission_window_millis).ok()?,
            declared: profile.bridge.max_candidate_work,
            available: profile
                .bridge
                .complete_message_bytes
                .checked_add(profile.bridge.control_message_bytes)?,
            scheduler: ObservationScheduler::new(
                RuntimeClock(started),
                CollectionPolicyLimits::new(
                    u64::try_from(profile.rate_interval_millis).ok()?,
                    profile.bridge.max_candidate_work,
                    profile.bridge.max_candidate_work,
                    profile.heavy_permits,
                )?,
                TransportPolicyLimits::new(
                    profile.bridge.complete_message_bytes,
                    profile.bridge.control_message_bytes,
                )?,
                SchedulerSafetyLimits::new(1, profile.max_overrun_debt)?,
            ),
            active: None,
            received_work: 0,
        })
    }
    pub fn admit(&mut self, key: &str) -> bool {
        if self.started.elapsed().as_millis() >= self.window || self.active.is_some() {
            return false;
        }
        let Some(id) = CollectionIntentId::new(key) else {
            return false;
        };
        let class = if key == "ship_core" {
            CollectionClass::Core
        } else {
            CollectionClass::Detail
        };
        let Some(intent) =
            CollectionIntent::new(id.clone(), class, WorkKind::Heavy, self.declared, 0)
        else {
            return false;
        };
        if self.scheduler.enqueue(intent).is_err() {
            return false;
        }
        let outcome = self
            .scheduler
            .deliver_pulse(DeliveredPulse::new(self.available));
        if outcome.admission().is_none() {
            return false;
        }
        self.active = Some(id);
        self.received_work = 0;
        true
    }
    pub const fn received(&mut self, bytes: usize) {
        self.received_work = self.received_work.saturating_add(bytes);
    }
    pub fn complete(&mut self) {
        if let Some(id) = self.active.take() {
            let _ = self.scheduler.complete(&id, self.received_work);
        }
    }
}
