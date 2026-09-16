use core::ffi::{c_int, c_void};

use crate::abi::{API, PRODUCER, TRANSPORT, bytes, integer, push_code};
use crate::lua_producer_context::{context, with_producer};
use crate::lua_progress::{error_code, outcome_code};
use crate::lua_transport::error_code_for_transport;
use crate::{ProducerOutcome, TransportPoll, TransportSendOutcome};

pub unsafe extern "C" fn begin_section(state: *mut c_void) -> c_int {
    let Some((api, handle)) = (unsafe { context(state) }) else {
        return invalid(state);
    };
    let source = PRODUCER
        .lock()
        .ok()
        .and_then(|producer| producer.as_ref().map(|value| value.source().clone()));
    let Some(input) =
        source.and_then(|source| unsafe { crate::lua_input::begin(api, state, &source) })
    else {
        return unsafe { push_code(api, state, -20) };
    };
    let result = with_producer(handle, |producer| match input {
        crate::lua_input::BeginInput::Clock(evidence) => producer.begin_section(evidence),
        crate::lua_input::BeginInput::ShipCore {
            evidence,
            expected_records,
        } => producer.begin_ship_section(evidence, expected_records),
    });
    unsafe { push_code(api, state, result.map_or_else(error_code, |()| 0)) }
}

pub unsafe extern "C" fn push_record(state: *mut c_void) -> c_int {
    let Some((api, handle)) = (unsafe { context(state) }) else {
        return invalid(state);
    };
    let Some(input) = (unsafe { crate::lua_input::record(api, state) }) else {
        return unsafe { push_code(api, state, -20) };
    };
    let result = with_producer(handle, |producer| match input {
        crate::lua_input::RecordInput::Clock(fact) => producer.push_record(&fact),
        crate::lua_input::RecordInput::ShipCore(record) => producer.push_ship_core(&record),
    });
    unsafe { push_code(api, state, result.map_or_else(error_code, |()| 0)) }
}

pub unsafe extern "C" fn finish_section(state: *mut c_void) -> c_int {
    let Some((api, handle)) = (unsafe { context(state) }) else {
        return invalid(state);
    };
    let Some(evidence) = (unsafe { crate::lua_input::finish(api, state) }) else {
        return unsafe { push_code(api, state, -20) };
    };
    let result = with_producer(handle, |producer| producer.finish_section(evidence));
    unsafe { push_code(api, state, result.map_or_else(error_code, |()| 0)) }
}

pub unsafe extern "C" fn fail_section(state: *mut c_void) -> c_int {
    let Some((api, handle)) = (unsafe { context(state) }) else {
        return invalid(state);
    };
    let Some(reason) = (unsafe { bytes(api, state, 2, 64) }) else {
        return unsafe { push_code(api, state, -20) };
    };
    if reason.is_empty() {
        return unsafe { push_code(api, state, -20) };
    }
    let result = with_producer(handle, |producer| {
        producer.fail_section();
        Ok(())
    });
    unsafe { push_code(api, state, result.map_or_else(error_code, |()| 0)) }
}

pub unsafe extern "C" fn progress(state: *mut c_void) -> c_int {
    let Some((api, handle)) = (unsafe { context(state) }) else {
        return invalid(state);
    };
    let Some(work) = (unsafe { integer(api, state, 2) }) else {
        return unsafe { push_code(api, state, -20) };
    };
    let Ok(mut producer_guard) = PRODUCER.lock() else {
        return unsafe { push_code(api, state, -18) };
    };
    let Some(producer) = producer_guard.as_mut() else {
        return unsafe { push_code(api, state, -18) };
    };
    let Ok(transport_guard) = TRANSPORT.lock() else {
        return unsafe { push_code(api, state, -18) };
    };
    let Some(transport) = transport_guard.as_ref() else {
        return unsafe { push_code(api, state, -18) };
    };
    let snapshot = transport.snapshot();
    let Some(now) = snapshot.monotonic_millis else {
        return unsafe { push_code(api, state, -22) };
    };
    if producer
        .observe_connection(snapshot.connection_generation, now)
        .is_err()
    {
        return unsafe { push_code(api, state, -18) };
    }
    if producer.pending_bytes().is_none() {
        match producer.progress(work, now) {
            Ok(ProducerOutcome::Progress) => {}
            result => {
                let code = result.map_or_else(error_code, outcome_code);
                return unsafe {
                    crate::lua_progress::push(api, state, code, producer, transport, now)
                };
            }
        }
    }
    let Some(message) = producer.pending_bytes() else {
        return unsafe { push_code(api, state, 0) };
    };
    let code = match transport.try_send(handle, message) {
        TransportSendOutcome::LocalHandoff => producer
            .mark_local_handoff(now)
            .map_or_else(error_code, |()| 1),
        TransportSendOutcome::CapacityUnavailable => 2,
        TransportSendOutcome::Rejected(error) => error_code_for_transport(error),
    };
    unsafe { crate::lua_progress::push(api, state, code, producer, transport, now) }
}

pub unsafe extern "C" fn poll_control(state: *mut c_void) -> c_int {
    let Some((api, handle)) = (unsafe { context(state) }) else {
        return invalid(state);
    };
    let Ok(mut producer_guard) = PRODUCER.lock() else {
        return unsafe { push_code(api, state, -18) };
    };
    let Some(producer) = producer_guard.as_mut() else {
        return unsafe { push_code(api, state, -18) };
    };
    let Ok(transport_guard) = TRANSPORT.lock() else {
        return unsafe { push_code(api, state, -18) };
    };
    let Some(transport) = transport_guard.as_ref() else {
        return unsafe { push_code(api, state, -18) };
    };
    let snapshot = transport.snapshot();
    let Some(now) = snapshot.monotonic_millis else {
        return unsafe { push_code(api, state, -22) };
    };
    if producer
        .observe_connection(snapshot.connection_generation, now)
        .is_err()
    {
        return unsafe { push_code(api, state, -18) };
    }
    let code = match transport.poll_control(handle, 512) {
        TransportPoll::Message(bytes) => {
            if transport.snapshot().connection_generation != snapshot.connection_generation {
                return unsafe { push_code(api, state, 3) };
            }
            let outcome = producer
                .apply_control(&bytes, now)
                .map_or_else(error_code, outcome_code);
            if outcome == outcome_code(ProducerOutcome::Disconnected) {
                let _ = transport.request_reconnect(handle);
            }
            outcome
        }
        TransportPoll::NoMessage => 3,
        TransportPoll::Closed => 9,
        TransportPoll::Rejected(error) => error_code_for_transport(error),
    };
    unsafe { push_code(api, state, code) }
}

fn invalid(state: *mut c_void) -> c_int {
    API.get()
        .copied()
        .map_or(0, |api| unsafe { push_code(api, state, -20) })
}

#[cfg(all(test, windows))]
#[path = "lua_producer_operations_tests.rs"]
mod tests;
