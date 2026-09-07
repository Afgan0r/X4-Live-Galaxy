#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_void;

mod abi;
mod abi_windows;
mod abi_windows_access;
mod abi_windows_io;
mod abi_windows_peer;
mod abi_windows_security;
mod abi_windows_sid;
mod handle;
mod lua_input;
mod lua_open;
mod lua_operations;
mod lua_producer_operations;
mod lua_table;
mod lua_transport;
mod producer;
mod producer_collection;
mod producer_feedback;
mod producer_message;
mod producer_types;
mod transport;
mod transport_peer;
mod transport_types;
mod transport_worker;
mod types;

pub use handle::HandleRegistry;
pub use producer::Producer;
pub use producer_types::{
    ProducerError, ProducerFeedback, ProducerLimits, ProducerOutcome, ProducerSource,
    ProducerState, SectionEvidence, SectionFinishEvidence, TypedFact,
};
pub use transport::NativeTransport;
pub use transport_peer::BridgePeer;
pub use transport_types::{
    CloseProgress, SecurityControl, SecurityEvidence, TransportConfig, TransportError,
    TransportPoll, TransportSendOutcome, WorkerSnapshot,
};
pub use types::{
    CarrierError, CarrierLimits, CloseOutcome, ControlPollOutcome, HandleToken, OpenConfig,
    SendOutcome,
};

pub const ABI_VERSION: u32 = 2;
pub const OPERATION_UNAVAILABLE: i32 = -100;
const OPERATIONS: [&str; 10] = [
    "abi_version",
    "open",
    "begin_section",
    "push_record",
    "finish_section",
    "fail_section",
    "progress",
    "poll_control",
    "reset",
    "close",
];

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModuleContract {
    pub abi_version: u32,
    pub operations: &'static [&'static str],
}

#[must_use]
pub const fn module_contract() -> ModuleContract {
    ModuleContract {
        abi_version: ABI_VERSION,
        operations: &OPERATIONS,
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum InitializerError {
    NullState,
    MissingHostModule,
    MissingHostSymbol(&'static str),
}

pub fn require_lua_symbols(available: impl FnMut(&str) -> bool) -> Result<(), InitializerError> {
    abi::require_lua_symbols(available)
}

#[must_use]
pub fn registered_operations() -> Vec<&'static str> {
    lua_operations::REGISTRATIONS
        .iter()
        .filter_map(|(name, _)| core::str::from_utf8(&name[..name.len() - 1]).ok())
        .collect()
}

#[must_use]
pub fn contained_status(operation: impl FnOnce() -> i32 + std::panic::UnwindSafe) -> i32 {
    std::panic::catch_unwind(operation).unwrap_or(-1)
}

#[unsafe(no_mangle)]
#[expect(
    clippy::not_unsafe_ptr_arg_deref,
    reason = "Lua owns and supplies the opaque state pointer to its C initializer"
)]
pub extern "C" fn luaopen_live_galaxy_carrier(state: *mut c_void) -> i32 {
    contained_status(|| {
        // SAFETY: the Lua loader owns state for this call; initialize validates it.
        unsafe { abi::initialize(state) }.unwrap_or(0)
    })
    .max(0)
}
