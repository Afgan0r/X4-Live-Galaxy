use core::ffi::{c_int, c_void};
use std::num::NonZeroUsize;

use crate::abi_windows::LuaFn;
use crate::lua_producer_operations::{
    begin_section as begin_section_inner, fail_section as fail_section_inner,
    finish_section as finish_section_inner, poll_control as poll_control_inner,
    progress as progress_inner, push_record as push_record_inner,
};
use crate::{
    ABI_VERSION, CarrierLimits, NativeTransport, OpenConfig, Producer, ProducerSource,
    TransportConfig,
    abi::{API, PRODUCER, REGISTRY, TRANSPORT, integer, push_code},
};

pub const REGISTRATIONS: [(&[u8], LuaFn); 10] = [
    (b"abi_version\0", abi_version),
    (b"open\0", open),
    (b"begin_section\0", begin_section),
    (b"push_record\0", push_record),
    (b"finish_section\0", finish_section),
    (b"fail_section\0", fail_section),
    (b"progress\0", progress),
    (b"poll_control\0", poll_control),
    (b"reset\0", reset),
    (b"close\0", close),
];

macro_rules! guarded {
    ($name:ident, $inner:ident) => {
        unsafe extern "C" fn $name(state: *mut c_void) -> c_int {
            guard(state, || unsafe { $inner(state) })
        }
    };
}
guarded!(begin_section, begin_section_inner);
guarded!(push_record, push_record_inner);
guarded!(finish_section, finish_section_inner);
guarded!(fail_section, fail_section_inner);
guarded!(progress, progress_inner);
guarded!(poll_control, poll_control_inner);
guarded!(abi_version, abi_version_inner);
guarded!(open, open_inner);
guarded!(reset, reset_inner);
guarded!(close, close_inner);

const PIPE_ENDPOINT: &str = r"\\.\pipe\live_galaxy";

unsafe fn abi_version_inner(state: *mut c_void) -> c_int {
    let Some(api) = API.get().copied() else {
        return 0;
    };
    unsafe { push_code(api, state, isize::try_from(ABI_VERSION).unwrap_or_default()) }
}

unsafe fn open_inner(state: *mut c_void) -> c_int {
    let Some(api) = API.get().copied() else {
        return 0;
    };
    if unsafe { integer(api, state, 1) } != Some(ABI_VERSION as usize) {
        return unsafe { push_code(api, state, -10) };
    }
    let Some(args) = (unsafe { crate::lua_open::decode(api, state) }) else {
        return unsafe { push_code(api, state, -20) };
    };
    let (Some(data), Some(control)) = (
        NonZeroUsize::new(args.limits.data_message_bytes),
        NonZeroUsize::new(args.limits.control_message_bytes),
    ) else {
        return unsafe { push_code(api, state, -20) };
    };
    let config = OpenConfig::current(CarrierLimits::new(data, control));
    let result = REGISTRY.lock().ok().and_then(|mut registry| {
        crate::lua_generation::initialize(&mut registry).ok()?;
        registry.open(config).ok()
    });
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
    let source = ProducerSource {
        session_id: format!("x4-session-{}", token.generation),
        producer_incarnation: format!("x4-producer-{}", token.generation),
        transport_epoch: u64::from(token.generation),
        source_scope: args.source_scope,
        source_epoch_status: args.epoch_status,
        source_boundary: args.boundary,
    };
    let producer = Producer::new(args.limits, source, 0);
    let (Ok(transport), Ok(producer)) = (transport, producer) else {
        let _closed = REGISTRY.lock().map(|mut registry| registry.close(token));
        return unsafe { push_code(api, state, -18) };
    };
    let (Ok(mut active_transport), Ok(mut active_producer)) = (TRANSPORT.lock(), PRODUCER.lock())
    else {
        let _closed = REGISTRY.lock().map(|mut registry| registry.close(token));
        return unsafe { push_code(api, state, -18) };
    };
    *active_transport = Some(transport);
    *active_producer = Some(producer);
    unsafe { (api.push_integer)(state, 0) };
    let encoded = format!("{}:{}", token.generation, token.slot);
    unsafe { (api.push_string)(state, encoded.as_ptr().cast(), encoded.len()) };
    2
}

unsafe fn reset_inner(state: *mut c_void) -> c_int {
    let result = crate::lua_transport::close_handle(state, false);
    clear_producer();
    result
}

unsafe fn close_inner(state: *mut c_void) -> c_int {
    let result = crate::lua_transport::close_handle(state, false);
    clear_producer();
    result
}

fn guard(state: *mut c_void, operation: impl FnOnce() -> c_int) -> c_int {
    std::panic::catch_unwind(std::panic::AssertUnwindSafe(operation)).unwrap_or_else(|_| {
        API.get()
            .copied()
            .map_or(0, |api| unsafe { push_code(api, state, -20) })
    })
}

fn clear_producer() {
    if let Ok(mut producer) = PRODUCER.lock() {
        *producer = None;
    }
}
