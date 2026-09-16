use core::ffi::{c_char, c_void};
use std::sync::Mutex;

use crate::abi_windows::LuaApi;

pub(crate) struct FakeLuaState {
    pub(crate) token: Vec<u8>,
    pub(crate) reason: Option<Vec<u8>>,
    pub(crate) pushed: Vec<isize>,
    pub(crate) pushed_strings: Vec<String>,
}

pub(crate) static ABI_TEST_LOCK: Mutex<()> = Mutex::new(());

pub(crate) fn fake_api() -> LuaApi {
    LuaApi {
        create_table: noop_table,
        push_closure: noop_closure,
        push_integer,
        push_string,
        set_field: noop_field,
        to_integer: noop_integer,
        to_string,
        get_top: noop_int,
        get_metatable: noop_index_int,
        lua_type,
        next: noop_index_int,
        push_nil: noop_state,
        raw_get: noop_index,
        set_top: noop_index,
        to_boolean: noop_index_int,
        to_number: noop_number,
    }
}

unsafe extern "C" fn push_integer(state: *mut c_void, value: isize) {
    unsafe { &mut *state.cast::<FakeLuaState>() }
        .pushed
        .push(value);
}

unsafe extern "C" fn push_string(state: *mut c_void, value: *const c_char, length: usize) {
    let bytes = unsafe { core::slice::from_raw_parts(value.cast::<u8>(), length) };
    unsafe { &mut *state.cast::<FakeLuaState>() }
        .pushed_strings
        .push(String::from_utf8_lossy(bytes).into_owned());
}

unsafe extern "C" fn to_string(
    state: *mut c_void,
    index: i32,
    length: *mut usize,
) -> *const c_char {
    let state = unsafe { &mut *state.cast::<FakeLuaState>() };
    let bytes = if index == 1 {
        &state.token
    } else {
        state.reason.as_ref().unwrap_or(&state.token)
    };
    unsafe { *length = bytes.len() };
    bytes.as_ptr().cast()
}

unsafe extern "C" fn lua_type(state: *mut c_void, index: i32) -> i32 {
    let state = unsafe { &*state.cast::<FakeLuaState>() };
    i32::from(index == 1 || (index == 2 && state.reason.is_some())) * 4
}

unsafe extern "C" fn noop_table(_: *mut c_void, _: i32, _: i32) {}
unsafe extern "C" fn noop_closure(_: *mut c_void, _: Option<crate::abi_windows::LuaFn>, _: i32) {}
unsafe extern "C" fn noop_field(_: *mut c_void, _: i32, _: *const c_char) {}
unsafe extern "C" fn noop_integer(_: *mut c_void, _: i32) -> isize {
    0
}
unsafe extern "C" fn noop_int(_: *mut c_void) -> i32 {
    0
}
unsafe extern "C" fn noop_index_int(_: *mut c_void, _: i32) -> i32 {
    0
}
unsafe extern "C" fn noop_state(_: *mut c_void) {}
unsafe extern "C" fn noop_index(_: *mut c_void, _: i32) {}
unsafe extern "C" fn noop_number(_: *mut c_void, _: i32) -> f64 {
    0.0
}
