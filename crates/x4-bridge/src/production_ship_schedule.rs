use crate::HeavyShipLimits;
use std::time::Instant;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShipAdmissionDenial {
    WindowExhausted,
    SelectionUnavailable,
}

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
        self.try_admit(key).is_ok()
    }

    pub(crate) fn try_admit(&mut self, key: &str) -> Result<(), ShipAdmissionDenial> {
        self.admit_at(key, Instant::now())
    }

    fn admit_at(&mut self, key: &str, now: Instant) -> Result<(), ShipAdmissionDenial> {
        if self.active.is_some() || key.is_empty() || key.len() > 64 {
            return Err(ShipAdmissionDenial::SelectionUnavailable);
        }
        if now.saturating_duration_since(self.started).as_millis() >= self.window {
            return Err(ShipAdmissionDenial::WindowExhausted);
        }
        self.active = Some(key.to_owned());
        self.received_work = 0;
        Ok(())
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

    #[test]
    fn the_cutoff_only_closes_admission_for_an_otherwise_valid_selection() {
        use std::time::Duration;

        let limits =
            HeavyShipLimits::parse(include_str!("../../../config/heavy-ship-experiment.json"))
                .expect("heavy limits");
        let mut schedule = ShipSchedule::new(&limits).expect("valid schedule");
        let deadline = schedule.started
            + Duration::from_millis(u64::try_from(schedule.window).expect("window"));
        let key = "a".repeat(64);
        let too_long = format!("{key}a");
        let just_before = deadline
            .checked_sub(Duration::from_millis(1))
            .expect("millisecond before cutoff");
        assert_eq!(
            schedule.admit_at(&too_long, just_before),
            Err(ShipAdmissionDenial::SelectionUnavailable)
        );
        assert_eq!(schedule.admit_at(&key, just_before), Ok(()));
        assert_eq!(
            schedule.admit_at(&key, deadline),
            Err(ShipAdmissionDenial::SelectionUnavailable)
        );
        schedule.complete();

        assert_eq!(
            schedule.admit_at("", deadline),
            Err(ShipAdmissionDenial::SelectionUnavailable)
        );
        assert_eq!(
            schedule.admit_at(&key, deadline),
            Err(ShipAdmissionDenial::WindowExhausted)
        );
        assert_eq!(
            schedule.admit_at(&key, deadline + Duration::from_millis(1)),
            Err(ShipAdmissionDenial::WindowExhausted)
        );
        assert_eq!(schedule.active, None);
    }
}
