use core::ffi::{c_int, c_void};
use std::sync::{Mutex, OnceLock};

use crate::{
    CarrierError, HandleRegistry, HandleToken, InitializerError,
    abi_windows::{LuaApi, LuaFn},
};

const REQUIRED: [&str; 7] = [
    "lua_createtable",
    "lua_pushcclosure",
    "lua_pushinteger",
    "lua_pushlstring",
    "lua_setfield",
    "lua_tointeger",
    "lua_tolstring",
];

pub(crate) static API: OnceLock<LuaApi> = OnceLock::new();
pub(crate) static REGISTRY: Mutex<HandleRegistry> = Mutex::new(HandleRegistry::new());

pub fn require_lua_symbols(
    mut available: impl FnMut(&str) -> bool,
) -> Result<(), InitializerError> {
    for name in REQUIRED {
        if !available(name) {
            return Err(InitializerError::MissingHostSymbol(name));
        }
    }
    Ok(())
}

#[must_use]
pub fn copy_owned(bytes: &[u8]) -> Vec<u8> {
    bytes.to_vec()
}

pub(crate) unsafe fn integer(api: LuaApi, state: *mut c_void, index: c_int) -> Option<usize> {
    // SAFETY: state and stack index belong to the current Lua call.
    let value = unsafe { (api.to_integer)(state, index) };
    usize::try_from(value).ok()
}

pub(crate) unsafe fn bytes(api: LuaApi, state: *mut c_void, index: c_int) -> Option<Vec<u8>> {
    let mut length = 0;
    // SAFETY: Lua owns the pointer through this call; bytes are copied now.
    let pointer = unsafe { (api.to_string)(state, index, &raw mut length) };
    if pointer.is_null() {
        return None;
    }
    // SAFETY: lua_tolstring guarantees length readable bytes for this call.
    Some(unsafe { core::slice::from_raw_parts(pointer.cast(), length) }.to_vec())
}

pub(crate) fn error_code(error: CarrierError) -> isize {
    match error {
        CarrierError::WrongVersion => -10,
        CarrierError::WrongRecordSize => -11,
        CarrierError::MessageTooLarge => -12,
        CarrierError::ControlTooLarge => -13,
        CarrierError::ControlCapacityUnavailable => -14,
        CarrierError::InvalidOutputCapacity => -15,
        CarrierError::StaleHandle => -16,
        CarrierError::GenerationExhausted => -17,
    }
}

pub(crate) unsafe fn push_code(api: LuaApi, state: *mut c_void, code: isize) -> c_int {
    // SAFETY: state belongs to the active Lua call.
    unsafe { (api.push_integer)(state, code) };
    1
}

pub(crate) unsafe fn token(api: LuaApi, state: *mut c_void) -> Option<HandleToken> {
    // SAFETY: copies the token string from the active Lua stack.
    let raw = unsafe { bytes(api, state, 1) }?;
    HandleToken::decode(core::str::from_utf8(&raw).ok()?)
}

pub unsafe fn initialize(state: *mut c_void) -> Result<c_int, InitializerError> {
    if state.is_null() {
        return Err(InitializerError::NullState);
    }
    let api = if let Some(api) = API.get() {
        api
    } else {
        // SAFETY: resolver validates every required Lua 5.1 host symbol.
        let resolved = unsafe { crate::abi_windows::resolve()? };
        let _ignored_race = API.set(resolved);
        API.get().ok_or(InitializerError::MissingHostModule)?
    };
    // SAFETY: state is live for this initializer call.
    unsafe { (api.create_table)(state, 0, 7) };
    for (name, function) in crate::lua_operations::REGISTRATIONS {
        // SAFETY: closures retain no Lua state or caller data.
        unsafe {
            (api.push_closure)(state, Some(function as LuaFn), 0);
            (api.set_field)(state, -2, name.as_ptr().cast());
        }
    }
    Ok(1)
}
