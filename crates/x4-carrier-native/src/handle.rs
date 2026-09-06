use crate::{CarrierError, CloseOutcome, ControlPollOutcome, HandleToken, OpenConfig, SendOutcome};

pub struct HandleRegistry {
    next_generation: u32,
}

impl Default for HandleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl HandleRegistry {
    #[must_use]
    pub const fn new() -> Self {
        Self { next_generation: 1 }
    }

    #[must_use]
    pub const fn starting_at(next_generation: u32) -> Self {
        Self { next_generation }
    }

    pub fn open(&mut self, _config: OpenConfig) -> Result<HandleToken, CarrierError> {
        let token = HandleToken {
            generation: self.next_generation,
            slot: 1,
        };
        self.next_generation = self.next_generation.saturating_add(1);
        Ok(token)
    }

    pub fn try_send(&mut self, _token: HandleToken, bytes: &[u8]) -> SendOutcome {
        let _owned = crate::abi::copy_owned(bytes);
        SendOutcome::CapacityUnavailable
    }

    #[must_use]
    pub fn pending_copy(&self, _token: HandleToken) -> Option<Vec<u8>> {
        None
    }

    pub fn queue_control(
        &mut self,
        _token: HandleToken,
        _bytes: &[u8],
    ) -> Result<(), CarrierError> {
        Ok(())
    }

    pub fn poll_control(&mut self, _token: HandleToken, _capacity: usize) -> ControlPollOutcome {
        ControlPollOutcome::NoMessage
    }

    pub fn close(&mut self, _token: HandleToken) -> CloseOutcome {
        CloseOutcome::Closed
    }
}
