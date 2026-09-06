use core::num::NonZeroUsize;

use crate::ABI_VERSION;

pub const OPEN_CONFIG_RECORD_SIZE: u32 = 24;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CarrierLimits {
    pub data_message_bytes: NonZeroUsize,
    pub control_message_bytes: NonZeroUsize,
}

impl CarrierLimits {
    #[must_use]
    pub const fn new(
        data_message_bytes: NonZeroUsize,
        control_message_bytes: NonZeroUsize,
    ) -> Self {
        Self {
            data_message_bytes,
            control_message_bytes,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct OpenConfig {
    pub abi_version: u32,
    pub record_size: u32,
    pub limits: CarrierLimits,
}

impl OpenConfig {
    #[must_use]
    pub const fn current(limits: CarrierLimits) -> Self {
        Self {
            abi_version: ABI_VERSION,
            record_size: OPEN_CONFIG_RECORD_SIZE,
            limits,
        }
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct HandleToken {
    pub generation: u32,
    pub slot: u32,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CarrierError {
    WrongVersion,
    WrongRecordSize,
    MessageTooLarge,
    ControlTooLarge,
    ControlCapacityUnavailable,
    InvalidOutputCapacity,
    StaleHandle,
    GenerationExhausted,
}

impl HandleToken {
    #[must_use]
    pub fn encode(self) -> String {
        format!("{:08x}:{:08x}", self.generation, self.slot)
    }

    #[must_use]
    pub fn decode(text: &str) -> Option<Self> {
        let (generation, slot) = text.split_once(':')?;
        if generation.len() != 8 || slot.len() != 8 {
            return None;
        }
        Some(Self {
            generation: u32::from_str_radix(generation, 16).ok()?,
            slot: u32::from_str_radix(slot, 16).ok()?,
        })
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum SendOutcome {
    LocalHandoff,
    CapacityUnavailable,
    Rejected(CarrierError),
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ControlPollOutcome {
    NoMessage,
    Message(Vec<u8>),
    Rejected(CarrierError),
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CloseOutcome {
    Closed,
    AlreadyClosed,
    Rejected(CarrierError),
}
