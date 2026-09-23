use crate::HeavyShipLimits;
use std::time::Instant;

pub struct ShipSchedule {
    started: Instant,
    window: u128,
    active: Option<String>,
    received_work: usize,
    attempts: u64,
}
impl ShipSchedule {
    #[must_use]
    pub fn new(profile: &HeavyShipLimits) -> Option<Self> {
        let started = Instant::now();
        Some(Self {
            started,
            window: u128::try_from(profile.admission_window_millis).ok()?,
            active: None,
            received_work: 0,
            attempts: 0,
        })
    }
    pub fn admit(&mut self, key: &str) -> bool {
        if self.started.elapsed().as_millis() >= self.window || self.active.is_some() {
            return false;
        }
        if key.is_empty() || key.len() > 64 {
            return false;
        }
        self.active = Some(key.to_owned());
        self.received_work = 0;
        true
    }
    pub const fn received(&mut self, bytes: usize) {
        self.received_work = self.received_work.saturating_add(bytes);
    }
    pub fn complete(&mut self) {
        self.active = None;
        self.received_work = 0;
    }

    pub const fn next_attempt(&mut self) -> u64 {
        self.attempts = self.attempts.saturating_add(1);
        self.attempts
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selection_attempts_advance_across_scopes() {
        let limits =
            HeavyShipLimits::parse(include_str!("../../../config/heavy-ship-experiment.json"))
                .expect("heavy limits");
        let mut schedule = ShipSchedule::new(&limits).expect("valid schedule");
        assert_eq!(schedule.next_attempt(), 1);
        assert!(schedule.admit("ship_core:argon"));
        schedule.complete();
        assert_eq!(schedule.next_attempt(), 2);
        assert!(schedule.admit("ship_core:teladi"));
    }
}
