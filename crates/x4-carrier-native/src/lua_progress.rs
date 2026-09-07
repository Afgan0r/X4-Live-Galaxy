use core::ffi::{c_int, c_void};

use crate::abi_windows::LuaApi;
use crate::{NativeTransport, Producer, ProducerState};

pub unsafe fn push(
    api: LuaApi,
    state: *mut c_void,
    code: isize,
    producer: &Producer,
    transport: &NativeTransport,
    monotonic_millis: u64,
) -> c_int {
    let snapshot = transport.snapshot();
    let monotonic = monotonic_millis.to_string();
    let fields = [
        state_name(producer.state()),
        if snapshot.connected {
            "connected"
        } else if snapshot.closed {
            "closed"
        } else {
            "waiting"
        },
        monotonic.as_str(),
        if pending(producer.state()) {
            "occupied"
        } else {
            "available"
        },
        producer.source().producer_incarnation.as_str(),
    ];
    unsafe { (api.push_integer)(state, code) };
    for field in fields {
        unsafe { (api.push_string)(state, field.as_ptr().cast(), field.len()) };
    }
    6
}

const fn pending(state: ProducerState) -> bool {
    matches!(
        state,
        ProducerState::PendingStart
            | ProducerState::PendingBatch
            | ProducerState::PendingCompletion
            | ProducerState::PausedAfterFailure
    )
}

const fn state_name(state: ProducerState) -> &'static str {
    match state {
        ProducerState::AwaitingCompatibility => "awaiting_compatibility",
        ProducerState::Ready => "ready",
        ProducerState::SectionReserved => "section_reserved",
        ProducerState::Collecting => "collecting",
        ProducerState::PendingStart => "pending_start",
        ProducerState::PendingBatch => "pending_batch",
        ProducerState::PendingCompletion => "pending_completion",
        ProducerState::PausedAfterFailure => "paused_after_failure",
        ProducerState::Incompatible => "incompatible",
        ProducerState::Closed => "closed",
    }
}
