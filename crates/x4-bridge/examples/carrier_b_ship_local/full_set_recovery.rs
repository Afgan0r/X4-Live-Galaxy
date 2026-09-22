use std::time::Instant;

use observation_ingest::CarrierIdentity;
use x4_bridge::{
    HeavyShipLimits, ProductionObservationSession, ReceiveProgress, RecoveryState, ShipSchedule,
    recover_idle,
};
use x4_carrier_native::BridgePeer;

use super::Result;

pub fn recover(
    peer: &mut BridgePeer,
    identity: &CarrierIdentity,
    profile: &HeavyShipLimits,
    receiver: &mut ProductionObservationSession,
    next: &mut String,
) -> Result<bool> {
    let mut schedule = ShipSchedule::new(profile);
    if !schedule.as_mut().is_some_and(|active| active.admit(next)) {
        return Err("blocked recovery schedule".into());
    }
    let mut progress = ReceiveProgress::issued(Instant::now(), &profile.bridge, true);
    let recovered = recover_idle(
        peer,
        identity,
        &profile.bridge,
        receiver,
        &mut schedule,
        RecoveryState {
            key: next,
            progress: &mut progress,
            monotonic_millis: u64::MAX,
            issued_at: Instant::now(),
        },
    );
    recovered
        .then_some(true)
        .ok_or_else(|| "blocked recovery path refused stale scope".into())
}
