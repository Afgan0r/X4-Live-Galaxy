use core::ffi::{c_int, c_void};

use crate::abi_windows::LuaApi;
use crate::{NativeTransport, Producer, ProducerError, ProducerOutcome, ProducerState};

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
    let capacity = format!(
        "{}:{}",
        if producer.collection_admitted() {
            "available"
        } else {
            "occupied"
        },
        snapshot.pending_operation_owners
    );
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
        capacity.as_str(),
        producer.source().producer_incarnation.as_str(),
    ];
    unsafe { (api.push_integer)(state, code) };
    for field in fields {
        unsafe { (api.push_string)(state, field.as_ptr().cast(), field.len()) };
    }
    6
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

pub const fn error_code(error: ProducerError) -> isize {
    match error {
        ProducerError::DataLimit => -12,
        ProducerError::ControlLimit => -13,
        ProducerError::StaleEpoch => -16,
        ProducerError::Incompatible => 10,
        ProducerError::ClockUnavailable => -22,
        ProducerError::InvalidInput => -20,
        ProducerError::InvalidTransition => -21,
    }
}

pub const fn outcome_code(outcome: ProducerOutcome) -> isize {
    match outcome {
        ProducerOutcome::Accepted => 0,
        ProducerOutcome::Progress => 1,
        ProducerOutcome::CapacityUnavailable => 2,
        ProducerOutcome::NoControl => 3,
        ProducerOutcome::Received => 4,
        ProducerOutcome::Committed => 5,
        ProducerOutcome::PermanentlyRejected => 6,
        ProducerOutcome::Ambiguous => 7,
        ProducerOutcome::PausedAfterFailure => 8,
        ProducerOutcome::Disconnected => 9,
        ProducerOutcome::RestartRequired => 10,
    }
}
