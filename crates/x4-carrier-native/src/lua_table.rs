use core::ffi::{c_int, c_void};

use crate::abi_windows::LuaApi;

pub unsafe fn exact_keys(api: LuaApi, state: *mut c_void, index: c_int, expected: &[&str]) -> bool {
    let table = unsafe { absolute(api, state, index) };
    if unsafe { (api.lua_type)(state, table) } != 5 {
        return false;
    }
    if unsafe { (api.get_metatable)(state, table) } != 0 {
        unsafe { pop(api, state) };
        return false;
    }
    let mut count = 0;
    unsafe { (api.push_nil)(state) };
    while unsafe { (api.next)(state, table) } != 0 {
        let valid = unsafe { strict_string(api, state, -2, 64) }
            .and_then(|bytes| core::str::from_utf8(&bytes).ok().map(str::to_owned))
            .is_some_and(|key| expected.contains(&key.as_str()));
        unsafe { pop(api, state) };
        if !valid {
            unsafe { pop(api, state) };
            return false;
        }
        count += 1;
    }
    count == expected.len()
}

pub unsafe fn field_string(
    api: LuaApi,
    state: *mut c_void,
    index: c_int,
    key: &str,
    limit: usize,
) -> Option<String> {
    let table = unsafe { absolute(api, state, index) };
    unsafe { push_key(api, state, key) };
    unsafe { (api.raw_get)(state, table) };
    let value = unsafe { strict_string(api, state, -1, limit) }
        .and_then(|bytes| String::from_utf8(bytes).ok());
    unsafe { pop(api, state) };
    value
}

pub unsafe fn field_integer(
    api: LuaApi,
    state: *mut c_void,
    index: c_int,
    key: &str,
) -> Option<usize> {
    let table = unsafe { absolute(api, state, index) };
    unsafe { push_key(api, state, key) };
    unsafe { (api.raw_get)(state, table) };
    let value = unsafe { crate::abi::integer(api, state, -1) };
    unsafe { pop(api, state) };
    value
}

pub unsafe fn field_bool(api: LuaApi, state: *mut c_void, index: c_int, key: &str) -> Option<bool> {
    let table = unsafe { absolute(api, state, index) };
    unsafe { push_key(api, state, key) };
    unsafe { (api.raw_get)(state, table) };
    let value = (unsafe { (api.lua_type)(state, -1) } == 1)
        .then(|| unsafe { (api.to_boolean)(state, -1) != 0 });
    unsafe { pop(api, state) };
    value
}

unsafe fn strict_string(
    api: LuaApi,
    state: *mut c_void,
    index: c_int,
    limit: usize,
) -> Option<Vec<u8>> {
    if unsafe { (api.lua_type)(state, index) } != 4 {
        return None;
    }
    let mut length = 0;
    let pointer = unsafe { (api.to_string)(state, index, &raw mut length) };
    if pointer.is_null() || length > limit {
        return None;
    }
    Some(unsafe { core::slice::from_raw_parts(pointer.cast(), length) }.to_vec())
}

unsafe fn absolute(api: LuaApi, state: *mut c_void, index: c_int) -> c_int {
    if index > 0 {
        index
    } else {
        (unsafe { (api.get_top)(state) }) + index + 1
    }
}

unsafe fn push_key(api: LuaApi, state: *mut c_void, key: &str) {
    unsafe { (api.push_string)(state, key.as_ptr().cast(), key.len()) };
}

unsafe fn pop(api: LuaApi, state: *mut c_void) {
    unsafe { (api.set_top)(state, -2) };
}
