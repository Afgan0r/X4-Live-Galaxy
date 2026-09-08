use std::sync::atomic::Ordering;
use std::time::{Duration, Instant};

use super::NativeTransport;

impl NativeTransport {
    pub fn set_clock_for_test(&self, monotonic_millis: Option<u64>) -> bool {
        let Ok(_guard) = self.shared.clock_test_lock.lock() else {
            return false;
        };
        self.shared.clock_test_override.store(
            if monotonic_millis.is_some() { 2 } else { 1 },
            Ordering::Release,
        );
        if let Some(value) = monotonic_millis {
            self.shared.millis.store(value, Ordering::Release);
        }
        self.shared
            .clock_available
            .store(monotonic_millis.is_some(), Ordering::Release);
        true
    }

    pub fn wait_for_control_for_test(&self, timeout: Duration) -> bool {
        let deadline = Instant::now() + timeout;
        while Instant::now() < deadline && !self.capture_control_for_test() {
            std::thread::yield_now();
        }
        self.capture_control_for_test()
    }

    fn capture_control_for_test(&self) -> bool {
        let Ok(mut pending) = self.control_pending.lock() else {
            return false;
        };
        if pending.is_some() {
            return true;
        }
        let control = self
            .control_rx
            .lock()
            .ok()
            .and_then(|receiver| receiver.try_recv().ok());
        *pending = control;
        pending.is_some()
    }
}
