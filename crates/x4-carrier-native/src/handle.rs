use crate::{CarrierError, CloseOutcome, ControlPollOutcome, HandleToken, OpenConfig, SendOutcome};

pub struct HandleRegistry {
    next_generation: Option<u32>,
    current: Option<Entry>,
}

struct Entry {
    token: HandleToken,
    config: OpenConfig,
    active: bool,
    pending: Option<Vec<u8>>,
    control: Option<Vec<u8>>,
}

impl Default for HandleRegistry {
    fn default() -> Self {
        Self::new()
    }
}

impl HandleRegistry {
    #[must_use]
    pub const fn new() -> Self {
        Self {
            next_generation: Some(1),
            current: None,
        }
    }

    #[must_use]
    pub const fn starting_at(next_generation: u32) -> Self {
        Self {
            next_generation: Some(next_generation),
            current: None,
        }
    }

    pub fn open(&mut self, config: OpenConfig) -> Result<HandleToken, CarrierError> {
        if config.abi_version != crate::ABI_VERSION {
            return Err(CarrierError::WrongVersion);
        }
        if config.record_size != crate::types::OPEN_CONFIG_RECORD_SIZE {
            return Err(CarrierError::WrongRecordSize);
        }
        let generation = self
            .next_generation
            .ok_or(CarrierError::GenerationExhausted)?;
        let token = HandleToken {
            generation,
            slot: 1,
        };
        self.next_generation = generation.checked_add(1);
        self.current = Some(Entry {
            token,
            config,
            active: true,
            pending: None,
            control: None,
        });
        Ok(token)
    }

    pub fn try_send(&mut self, token: HandleToken, bytes: &[u8]) -> SendOutcome {
        let Some(entry) = self.current.as_mut() else {
            return SendOutcome::Rejected(CarrierError::StaleHandle);
        };
        if bytes.len() > entry.config.limits.data_message_bytes.get() {
            return SendOutcome::Rejected(CarrierError::MessageTooLarge);
        }
        if entry.token != token || !entry.active {
            return SendOutcome::Rejected(CarrierError::StaleHandle);
        }
        if entry.pending.is_some() {
            return SendOutcome::CapacityUnavailable;
        }
        entry.pending = Some(crate::abi::copy_owned(bytes));
        SendOutcome::LocalHandoff
    }

    #[must_use]
    pub fn pending_copy(&self, token: HandleToken) -> Option<Vec<u8>> {
        self.current
            .as_ref()
            .filter(|entry| entry.active && entry.token == token)
            .and_then(|entry| entry.pending.clone())
    }

    pub fn queue_control(&mut self, token: HandleToken, bytes: &[u8]) -> Result<(), CarrierError> {
        let entry = self.active_mut(token)?;
        if bytes.len() > entry.config.limits.control_message_bytes.get() {
            return Err(CarrierError::ControlTooLarge);
        }
        if entry.control.is_some() {
            return Err(CarrierError::ControlCapacityUnavailable);
        }
        entry.control = Some(bytes.to_vec());
        Ok(())
    }

    pub fn poll_control(&mut self, token: HandleToken, capacity: usize) -> ControlPollOutcome {
        let Ok(entry) = self.active_mut(token) else {
            return ControlPollOutcome::Rejected(CarrierError::StaleHandle);
        };
        let Some(control) = entry.control.as_ref() else {
            return ControlPollOutcome::NoMessage;
        };
        if capacity < control.len() {
            return ControlPollOutcome::Rejected(CarrierError::InvalidOutputCapacity);
        }
        ControlPollOutcome::Message(entry.control.take().unwrap_or_default())
    }

    pub fn reset(&mut self, token: HandleToken) -> Result<(), CarrierError> {
        let entry = self.active_mut(token)?;
        entry.pending = None;
        entry.control = None;
        Ok(())
    }

    pub fn close(&mut self, token: HandleToken) -> CloseOutcome {
        let Some(entry) = self.current.as_mut() else {
            return CloseOutcome::Rejected(CarrierError::StaleHandle);
        };
        if entry.token != token {
            return CloseOutcome::Rejected(CarrierError::StaleHandle);
        }
        if !entry.active {
            return CloseOutcome::AlreadyClosed;
        }
        entry.active = false;
        entry.pending = None;
        entry.control = None;
        CloseOutcome::Closed
    }

    #[must_use]
    pub fn is_active(&self, token: HandleToken) -> bool {
        self.current
            .as_ref()
            .is_some_and(|entry| entry.active && entry.token == token)
    }

    fn active_mut(&mut self, token: HandleToken) -> Result<&mut Entry, CarrierError> {
        self.current
            .as_mut()
            .filter(|entry| entry.active && entry.token == token)
            .ok_or(CarrierError::StaleHandle)
    }
}
