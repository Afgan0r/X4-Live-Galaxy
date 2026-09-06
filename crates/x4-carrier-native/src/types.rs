use core::num::NonZeroUsize;

use crate::ABI_VERSION;

pub const OPEN_CONFIG_RECORD_SIZE: u32 = 24;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct CarrierLimits {
    pub data_message_bytes: NonZeroUsize,
    pub control_message_bytes: NonZeroUsize,
}

impl CarrierLimits {
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
    InvalidOutputCapacity,
    StaleHandle,
    GenerationExhausted,
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
