use core::ffi::{c_int, c_void};
use std::sync::{Mutex, OnceLock};

use crate::{
    CarrierError, HandleRegistry, HandleToken, InitializerError, NativeTransport, Producer,
    abi_windows::{LuaApi, LuaFn},
};

const REQUIRED: [&str; 16] = [
    "lua_createtable",
    "lua_getmetatable",
    "lua_gettop",
    "lua_next",
    "lua_pushcclosure",
    "lua_pushinteger",
    "lua_pushlstring",
    "lua_pushnil",
    "lua_rawget",
    "lua_setfield",
    "lua_settop",
    "lua_toboolean",
    "lua_tointeger",
    "lua_tolstring",
    "lua_tonumber",
    "lua_type",
];

pub(crate) static API: OnceLock<LuaApi> = OnceLock::new();
pub(crate) static REGISTRY: Mutex<HandleRegistry> = Mutex::new(HandleRegistry::new());
pub(crate) static TRANSPORT: Mutex<Option<NativeTransport>> = Mutex::new(None);
pub(crate) static PRODUCER: Mutex<Option<Producer>> = Mutex::new(None);

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
    if unsafe { (api.lua_type)(state, index) } != 3 {
        return None;
    }
    let value = unsafe { (api.to_number)(state, index) };
    if !value.is_finite()
        || value.fract() != 0.0
        || !(0.0..=9_007_199_254_740_991.0).contains(&value)
    {
        return None;
    }
    let integer = unsafe { (api.to_integer)(state, index) };
    let round_trip = integer.to_string().parse::<f64>().ok()?;
    ((round_trip - value).abs() <= f64::EPSILON).then(|| usize::try_from(integer).ok())?
}

pub(crate) unsafe fn bytes(
    api: LuaApi,
    state: *mut c_void,
    index: c_int,
    limit: usize,
) -> Option<Vec<u8>> {
    if unsafe { (api.lua_type)(state, index) } != 4 {
        return None;
    }
    let mut length = 0;
    // SAFETY: Lua owns the pointer through this call; bytes are copied now.
    let pointer = unsafe { (api.to_string)(state, index, &raw mut length) };
    if pointer.is_null() || length > limit {
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
        CarrierError::TransportUnavailable => -18,
    }
}

pub(crate) unsafe fn push_code(api: LuaApi, state: *mut c_void, code: isize) -> c_int {
    // SAFETY: state belongs to the active Lua call.
    unsafe { (api.push_integer)(state, code) };
    1
}

pub(crate) unsafe fn token(api: LuaApi, state: *mut c_void) -> Option<HandleToken> {
    // SAFETY: copies the token string from the active Lua stack.
    let raw = unsafe { bytes(api, state, 1, 32) }?;
    let text = core::str::from_utf8(&raw).ok()?;
    let (generation, slot) = text.split_once(':')?;
    if generation.is_empty()
        || slot.is_empty()
        || generation.len() > 10
        || slot.len() > 10
        || !generation.bytes().all(|byte| byte.is_ascii_digit())
        || !slot.bytes().all(|byte| byte.is_ascii_digit())
    {
        return None;
    }
    Some(HandleToken {
        generation: generation.parse().ok()?,
        slot: slot.parse().ok()?,
    })
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
    unsafe { (api.create_table)(state, 0, 10) };
    for (name, function) in crate::lua_operations::REGISTRATIONS {
        // SAFETY: closures retain no Lua state or caller data.
        unsafe {
            (api.push_closure)(state, Some(function as LuaFn), 0);
            (api.set_field)(state, -2, name.as_ptr().cast());
        }
    }
    Ok(1)
}
