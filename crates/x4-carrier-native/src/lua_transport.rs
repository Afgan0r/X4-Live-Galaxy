use core::ffi::{c_int, c_void};

use crate::{
    CloseOutcome, TransportError,
    abi::{API, REGISTRY, TRANSPORT, error_code, push_code, token},
};

pub fn error_code_for_transport(error: TransportError) -> isize {
    match error {
        TransportError::StaleGeneration => -16,
        TransportError::MessageTooLarge => -12,
        TransportError::InvalidOutputCapacity => -15,
        TransportError::InvalidConfig
        | TransportError::Unavailable
        | TransportError::SecurityPolicyFailed
        | TransportError::AccessProbeFailed
        | TransportError::PipeCreationFailed
        | TransportError::WorkerStartFailed => -18,
    }
}

pub fn close_handle(state: *mut c_void, reset: bool) -> c_int {
    let Some(api) = API.get().copied() else {
        return 0;
    };
    let Some(handle) = (unsafe { token(api, state) }) else {
        return unsafe { push_code(api, state, -20) };
    };
    if let Ok(transport) = TRANSPORT.lock()
        && let Some(item) = transport.as_ref()
    {
        let _progress = item.request_close(handle);
    }
    let code = REGISTRY.lock().map_or(-16, |mut registry| {
        if reset {
            return registry.reset(handle).map_or_else(error_code, |()| 0);
        }
        match registry.close(handle) {
            CloseOutcome::Closed | CloseOutcome::AlreadyClosed => 0,
            CloseOutcome::Rejected(error) => error_code(error),
        }
    });
    unsafe { push_code(api, state, code) }
}
