#![deny(unsafe_op_in_unsafe_fn)]

pub const ABI_VERSION: u32 = 0;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct ModuleContract {
    pub abi_version: u32,
    pub operations: &'static [&'static str],
}

#[must_use]
pub const fn module_contract() -> ModuleContract {
    ModuleContract {
        abi_version: ABI_VERSION,
        operations: &[],
    }
}

#[must_use]
pub fn contained_status(operation: impl FnOnce() -> i32 + std::panic::UnwindSafe) -> i32 {
    std::panic::catch_unwind(operation).unwrap_or(-1)
}

#[unsafe(no_mangle)]
pub extern "C" fn luaopen_live_galaxy_carrier(_state: *mut core::ffi::c_void) -> i32 {
    contained_status(|| 0)
}
