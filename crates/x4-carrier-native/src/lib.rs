#![deny(unsafe_op_in_unsafe_fn)]

use core::ffi::{c_char, c_int, c_void};

mod abi;
mod handle;
mod types;

pub use handle::HandleRegistry;
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
const REQUIRED_LUA_SYMBOLS: [&str; 4] = [
    "lua_createtable",
    "lua_pushcclosure",
    "lua_pushinteger",
    "lua_setfield",
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

pub fn require_lua_symbols(
    mut available: impl FnMut(&str) -> bool,
) -> Result<(), InitializerError> {
    for name in REQUIRED_LUA_SYMBOLS {
        if !available(name) {
            return Err(InitializerError::MissingHostSymbol(name));
        }
    }
    Ok(())
}

#[must_use]
pub fn contained_status(operation: impl FnOnce() -> i32 + std::panic::UnwindSafe) -> i32 {
    std::panic::catch_unwind(operation).unwrap_or(-1)
}

type LuaFunction = unsafe extern "C" fn(*mut c_void) -> c_int;
type LuaCreateTable = unsafe extern "C" fn(*mut c_void, c_int, c_int);
type LuaPushClosure = unsafe extern "C" fn(*mut c_void, Option<LuaFunction>, c_int);
type LuaPushInteger = unsafe extern "C" fn(*mut c_void, isize);
type LuaSetField = unsafe extern "C" fn(*mut c_void, c_int, *const c_char);

#[derive(Clone, Copy)]
struct LuaApi {
    create_table: LuaCreateTable,
    push_closure: LuaPushClosure,
    push_integer: LuaPushInteger,
    set_field: LuaSetField,
}

static LUA_API: std::sync::OnceLock<LuaApi> = std::sync::OnceLock::new();

unsafe extern "C" fn lua_abi_version(state: *mut c_void) -> c_int {
    let Some(api) = LUA_API.get() else { return 0 };
    // SAFETY: Lua supplies its live state to a registered C function.
    unsafe { (api.push_integer)(state, isize::try_from(ABI_VERSION).unwrap_or_default()) };
    1
}

unsafe extern "C" fn lua_unavailable(_state: *mut c_void) -> c_int {
    OPERATION_UNAVAILABLE
}

#[cfg(windows)]
unsafe fn resolve_lua_api() -> Result<LuaApi, InitializerError> {
    use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};

    // SAFETY: null requests the current executable module and changes no state.
    let host = unsafe { GetModuleHandleA(core::ptr::null()) };
    if host.is_null() {
        return Err(InitializerError::MissingHostModule);
    }
    macro_rules! symbol {
        ($name:literal, $ty:ty) => {{
            // SAFETY: the NUL-terminated name is valid for this call.
            let raw = unsafe { GetProcAddress(host, concat!($name, "\0").as_ptr()) }
                .ok_or(InitializerError::MissingHostSymbol($name))?;
            // SAFETY: the required Lua 5.1 host contract fixes this symbol's signature.
            unsafe { core::mem::transmute::<unsafe extern "system" fn() -> isize, $ty>(raw) }
        }};
    }
    Ok(LuaApi {
        create_table: symbol!("lua_createtable", LuaCreateTable),
        push_closure: symbol!("lua_pushcclosure", LuaPushClosure),
        push_integer: symbol!("lua_pushinteger", LuaPushInteger),
        set_field: symbol!("lua_setfield", LuaSetField),
    })
}

#[cfg(not(windows))]
unsafe fn resolve_lua_api() -> Result<LuaApi, InitializerError> {
    Err(InitializerError::MissingHostModule)
}

unsafe fn initialize(state: *mut c_void) -> Result<c_int, InitializerError> {
    if state.is_null() {
        return Err(InitializerError::NullState);
    }
    // SAFETY: symbol signatures are fixed by the Lua 5.1 host contract.
    let api = unsafe { resolve_lua_api()? };
    let api = LUA_API.get_or_init(|| api);
    // SAFETY: state is owned by Lua for the duration of this initializer call.
    unsafe {
        (api.create_table)(
            state,
            0,
            c_int::try_from(OPERATIONS.len()).unwrap_or_default(),
        )
    };
    for name in OPERATIONS {
        let function = if name == "abi_version" {
            lua_abi_version
        } else {
            lua_unavailable
        };
        // SAFETY: the closure contains no upvalues and does not retain the Lua state.
        unsafe { (api.push_closure)(state, Some(function), 0) };
        let mut field = name.as_bytes().to_vec();
        field.push(0);
        // SAFETY: field is NUL terminated and remains live through the call.
        unsafe { (api.set_field)(state, -2, field.as_ptr().cast()) };
    }
    Ok(1)
}

#[unsafe(no_mangle)]
pub extern "C" fn luaopen_live_galaxy_carrier(state: *mut c_void) -> i32 {
    contained_status(|| {
        // SAFETY: the Lua loader supplies the state; initialize validates null first.
        unsafe { initialize(state) }.unwrap_or(0)
    })
}
