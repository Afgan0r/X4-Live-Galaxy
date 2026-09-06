#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::c_void;

mod abi;
mod abi_windows;
mod handle;
mod lua_operations;
mod transport;
mod types;

pub use handle::HandleRegistry;
pub use transport::{
    BridgePeer, CloseProgress, NativeTransport, SecurityEvidence, TransportConfig, TransportError,
    TransportPoll, TransportSendOutcome, WorkerSnapshot,
};
pub use types::{
    CarrierError, CarrierLimits, CloseOutcome, ControlPollOutcome, HandleToken, OpenConfig,
    SendOutcome,
};

pub const ABI_VERSION: u32 = 1;
pub const OPERATION_UNAVAILABLE: i32 = -100;
const OPERATIONS: [&str; 7] = [
    "abi_version",
    "open",
    "connection",
    "try_send",
    "poll",
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
