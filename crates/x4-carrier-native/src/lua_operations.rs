use core::ffi::{c_int, c_void};
use std::num::NonZeroUsize;

use crate::{
    ABI_VERSION, CarrierLimits, NativeTransport, OpenConfig, TransportConfig, TransportError,
    TransportPoll, TransportSendOutcome,
    abi::{API, REGISTRY, TRANSPORT, bytes, integer, push_code, token},
    abi_windows::LuaFn,
    lua_transport::{close_handle, error_code_for_transport},
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

const PIPE_ENDPOINT: &str = r"\\.\pipe\live_galaxy";

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
    let transport = NativeTransport::start(
        TransportConfig {
            pipe_name: PIPE_ENDPOINT.to_owned(),
            max_data_message_bytes: data.get(),
            max_control_message_bytes: control.get(),
        },
        token,
    );
    let Ok(transport) = transport else {
        let _closed = REGISTRY.lock().map(|mut registry| registry.close(token));
        return unsafe { push_code(api, state, -18) };
    };
    let Ok(mut active_transport) = TRANSPORT.lock() else {
        let _closed = REGISTRY.lock().map(|mut registry| registry.close(token));
        return unsafe { push_code(api, state, -18) };
    };
    *active_transport = Some(transport);
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
            && TRANSPORT.lock().is_ok_and(|transport| {
                transport
                    .as_ref()
                    .is_some_and(|item| !item.snapshot().closed && item.snapshot().connected)
            })
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
    let outcome = TRANSPORT.lock().map_or(
        TransportSendOutcome::Rejected(TransportError::StaleGeneration),
        |transport| {
            transport.as_ref().map_or(
                TransportSendOutcome::Rejected(TransportError::Unavailable),
                |item| item.try_send(token, &message),
            )
        },
    );
    let code = match outcome {
        TransportSendOutcome::LocalHandoff => 1,
        TransportSendOutcome::CapacityUnavailable => 2,
        TransportSendOutcome::Rejected(error) => error_code_for_transport(error),
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
    let outcome = TRANSPORT.lock().map_or(
        TransportPoll::Rejected(TransportError::StaleGeneration),
        |transport| {
            transport.as_ref().map_or(
                TransportPoll::Rejected(TransportError::Unavailable),
                |item| item.poll_control(token, capacity),
            )
        },
    );
    match outcome {
        TransportPoll::Message(message) => {
            unsafe {
                (api.push_integer)(state, 1);
                (api.push_string)(state, message.as_ptr().cast(), message.len());
            }
            2
        }
        TransportPoll::NoMessage => unsafe { push_code(api, state, 0) },
        TransportPoll::Closed => unsafe { push_code(api, state, -18) },
        TransportPoll::Rejected(error) => unsafe {
            push_code(api, state, error_code_for_transport(error))
        },
    }
}

unsafe extern "C" fn reset(state: *mut c_void) -> c_int {
    close_handle(state, true)
}

unsafe extern "C" fn close(state: *mut c_void) -> c_int {
    close_handle(state, false)
}
