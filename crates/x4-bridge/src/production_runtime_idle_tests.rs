use super::ReceiveProgress;
use crate::HeavyShipLimits;
use std::time::{Duration, Instant};

fn progress(origin: Instant, heavy: bool) -> ReceiveProgress {
    let profile =
        HeavyShipLimits::parse(include_str!("../../../config/heavy-ship-experiment.json"))
            .expect("prepared profile");
    ReceiveProgress::issued(origin, &profile.bridge, heavy)
}

#[test]
fn heavy_initial_data_accepts_preparation_between_inactivity_and_age_deadlines() {
    let origin = Instant::now();
    let wait = progress(origin, true);
    assert!(!wait.expired(origin + Duration::from_millis(10_001)));
    assert!(!wait.expired(origin + Duration::from_millis(29_999)));
    assert!(wait.expired(origin + Duration::from_secs(30)));
    assert!(wait.expired(origin + Duration::from_millis(30_001)));
}

#[test]
fn heavy_polling_cannot_renew_preparation_deadline() {
    let origin = Instant::now();
    let wait = progress(origin, true);
    for millis in [1, 10_000, 20_000, 29_999] {
        assert!(!wait.expired(origin + Duration::from_millis(millis)));
    }
    assert!(wait.expired(origin + Duration::from_secs(30)));
}

#[test]
fn first_data_switches_to_unchanged_progress_inactivity() {
    let origin = Instant::now();
    let mut wait = progress(origin, true);
    let first_data = origin + Duration::from_secs(20);
    wait.received(first_data);
    assert!(!wait.expired(first_data + Duration::from_millis(9_999)));
    assert!(wait.expired(first_data + Duration::from_secs(10)));
    wait.received(first_data + Duration::from_secs(9));
    assert!(!wait.expired(first_data + Duration::from_millis(18_999)));
    assert!(wait.expired(first_data + Duration::from_secs(19)));
}

#[test]
fn newly_issued_selection_gets_its_own_finite_preparation_origin() {
    let origin = Instant::now();
    let mut wait = progress(origin, true);
    wait.received(origin + Duration::from_secs(5));
    assert!(wait.expired(origin + Duration::from_secs(15)));
    let issued = origin + Duration::from_millis(15_500);
    wait = progress(issued, true);
    assert!(!wait.expired(issued + Duration::from_millis(29_999)));
    assert!(wait.expired(issued + Duration::from_secs(30)));
}

#[test]
fn clock_profile_has_no_extended_preparation_phase() {
    let origin = Instant::now();
    let mut wait = progress(origin, false);
    assert!(!wait.expired(origin + Duration::from_millis(9_999)));
    assert!(wait.expired(origin + Duration::from_secs(10)));
    wait.received(origin + Duration::from_secs(5));
    assert!(wait.expired(origin + Duration::from_secs(15)));
}
