use crate::HeavyShipLimits;
use std::time::Instant;

pub struct ShipSchedule {
    started: Instant,
    window: u128,
    active: Option<String>,
    received_work: usize,
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
}
