use core::ffi::{c_int, c_void};
use std::num::NonZeroUsize;

use crate::{
    ABI_VERSION, CarrierError, CarrierLimits, CloseOutcome, ControlPollOutcome, HandleRegistry,
    HandleToken, OpenConfig, SendOutcome,
    abi::{API, REGISTRY, bytes, error_code, integer, push_code, token},
    abi_windows::LuaFn,
};

pub const REGISTRATIONS: [(&[u8], LuaFn); 7] = [
    (b"abi_version\0", abi_version),
    (b"open\0", open),
    (b"connection\0", connection),
    (b"try_send\0", try_send),
    (b"poll\0", poll),
    (b"reset\0", reset),
    (b"close\0", close),
];

unsafe extern "C" fn abi_version(state: *mut c_void) -> c_int {
    let Some(api) = API.get().copied() else {
        return 0;
    };
    // SAFETY: registered Lua entry receives a live state.
    unsafe { push_code(api, state, isize::try_from(ABI_VERSION).unwrap_or_default()) }
}

unsafe extern "C" fn open(state: *mut c_void) -> c_int {
    let Some(api) = API.get().copied() else {
        return 0;
    };
    // SAFETY: values are read from the active Lua stack and not retained.
    let values = unsafe {
        [
            integer(api, state, 1),
            integer(api, state, 2),
            integer(api, state, 3),
            integer(api, state, 4),
        ]
    };
    let [Some(version), Some(record), Some(data), Some(control)] = values else {
        return unsafe { push_code(api, state, -20) };
    };
    let (Some(data), Some(control)) = (NonZeroUsize::new(data), NonZeroUsize::new(control)) else {
        return unsafe { push_code(api, state, -20) };
    };
    let config = OpenConfig {
        abi_version: u32::try_from(version).unwrap_or_default(),
        record_size: u32::try_from(record).unwrap_or_default(),
        limits: CarrierLimits::new(data, control),
    };
    let result = REGISTRY
        .lock()
        .ok()
        .and_then(|mut registry| registry.open(config).ok());
    let Some(token) = result else {
        return unsafe { push_code(api, state, -17) };
    };
    // SAFETY: both values are pushed onto the active Lua stack.
    unsafe { (api.push_integer)(state, 0) };
    let encoded = token.encode();
    unsafe { (api.push_string)(state, encoded.as_ptr().cast(), encoded.len()) };
    2
}

unsafe extern "C" fn connection(state: *mut c_void) -> c_int {
    let Some(api) = API.get().copied() else {
        return 0;
    };
    let active = unsafe { token(api, state) }.is_some_and(|value| {
        REGISTRY
            .lock()
            .is_ok_and(|registry| registry.is_active(value))
    });
    unsafe { push_code(api, state, isize::from(active)) }
}

unsafe extern "C" fn try_send(state: *mut c_void) -> c_int {
    let Some(api) = API.get().copied() else {
        return 0;
    };
    let (Some(token), Some(message)) = (unsafe { token(api, state) }, unsafe {
        bytes(api, state, 2)
    }) else {
        return unsafe { push_code(api, state, -20) };
    };
    let outcome = REGISTRY.lock().map_or(
        SendOutcome::Rejected(CarrierError::StaleHandle),
        |mut registry| registry.try_send(token, &message),
    );
    let code = match outcome {
        SendOutcome::LocalHandoff => 1,
        SendOutcome::CapacityUnavailable => 2,
        SendOutcome::Rejected(error) => error_code(error),
    };
    unsafe { push_code(api, state, code) }
}

unsafe extern "C" fn poll(state: *mut c_void) -> c_int {
    let Some(api) = API.get().copied() else {
        return 0;
    };
    let (Some(token), Some(capacity)) = (unsafe { token(api, state) }, unsafe {
        integer(api, state, 2)
    }) else {
        return unsafe { push_code(api, state, -20) };
    };
    let outcome = REGISTRY.lock().map_or(
        ControlPollOutcome::Rejected(CarrierError::StaleHandle),
        |mut registry| registry.poll_control(token, capacity),
    );
    match outcome {
        ControlPollOutcome::Message(message) => {
            unsafe {
                (api.push_integer)(state, 1);
                (api.push_string)(state, message.as_ptr().cast(), message.len());
            }
            2
        }
        ControlPollOutcome::NoMessage => unsafe { push_code(api, state, 0) },
        ControlPollOutcome::Rejected(error) => unsafe { push_code(api, state, error_code(error)) },
    }
}

unsafe extern "C" fn reset(state: *mut c_void) -> c_int {
    mutate_handle(state, |registry, token| registry.reset(token).map(|()| 0))
}

unsafe extern "C" fn close(state: *mut c_void) -> c_int {
    mutate_handle(state, |registry, token| match registry.close(token) {
        CloseOutcome::Closed | CloseOutcome::AlreadyClosed => Ok(0),
        CloseOutcome::Rejected(error) => Err(error),
    })
}

fn mutate_handle(
    state: *mut c_void,
    operation: impl FnOnce(&mut HandleRegistry, HandleToken) -> Result<isize, CarrierError>,
) -> c_int {
    let Some(api) = API.get().copied() else {
        return 0;
    };
    let Some(token) = (unsafe { token(api, state) }) else {
        return unsafe { push_code(api, state, -20) };
    };
    let code = REGISTRY.lock().map_or(-16, |mut registry| {
        operation(&mut registry, token).unwrap_or_else(error_code)
    });
    unsafe { push_code(api, state, code) }
}
