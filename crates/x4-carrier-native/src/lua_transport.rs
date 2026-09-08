use core::ffi::{c_int, c_void};

use crate::{
    CloseOutcome, TransportError,
    abi::{API, PRODUCER, REGISTRY, TRANSPORT, error_code, push_code, token},
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
    let Ok(mut registry) = REGISTRY.lock() else {
        return unsafe { push_code(api, state, -16) };
    };
    let (code, owns_active) = {
        if reset {
            let result = registry.reset(handle);
            let code = result
                .as_ref()
                .map_or_else(|error| error_code(*error), |()| 0);
            (code, TeardownDecision::from_reset(result).allows())
        } else {
            let outcome = registry.close(handle);
            let code = match outcome {
                CloseOutcome::Closed | CloseOutcome::AlreadyClosed => 0,
                CloseOutcome::Rejected(error) => error_code(error),
            };
            (code, TeardownDecision::from_close(outcome).allows())
        }
    };
    if owns_active {
        teardown_globals(handle);
    }
    unsafe { push_code(api, state, code) }
}

#[derive(Clone, Copy)]
enum TeardownDecision {
    OwnsActive,
    PreserveReplacement,
}

impl TeardownDecision {
    const fn from_reset(result: Result<(), crate::CarrierError>) -> Self {
        if result.is_ok() {
            Self::OwnsActive
        } else {
            Self::PreserveReplacement
        }
    }

    const fn from_close(outcome: CloseOutcome) -> Self {
        if matches!(outcome, CloseOutcome::Closed) {
            Self::OwnsActive
        } else {
            Self::PreserveReplacement
        }
    }

    const fn allows(self) -> bool {
        matches!(self, Self::OwnsActive)
    }
}

fn teardown_globals(handle: crate::HandleToken) {
    if let Ok(mut transport) = TRANSPORT.lock() {
        if let Some(item) = transport.as_ref() {
            let _progress = item.request_close(handle);
        }
        *transport = None;
    }
    if let Ok(mut producer) = PRODUCER.lock() {
        *producer = None;
    }
}
