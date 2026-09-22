use core::ffi::{c_int, c_void};

use crate::abi::{API, PRODUCER, REGISTRY, push_code, token};
use crate::{HandleToken, Producer, ProducerError};

pub(crate) fn with_producer<T>(
    handle: HandleToken,
    operation: impl FnOnce(&mut Producer) -> Result<T, ProducerError>,
) -> Result<T, ProducerError> {
    if !REGISTRY
        .lock()
        .is_ok_and(|registry| registry.is_active(handle))
    {
        return Err(ProducerError::StaleEpoch);
    }
    PRODUCER
        .lock()
        .map_err(|_| ProducerError::InvalidTransition)?
        .as_mut()
        .ok_or(ProducerError::InvalidTransition)
        .and_then(operation)
}

pub(crate) unsafe fn context(
    state: *mut c_void,
) -> Option<(crate::abi_windows::LuaApi, HandleToken)> {
    let api = API.get().copied()?;
    let handle = unsafe { token(api, state) }?;
    Some((api, handle))
}

pub(crate) fn invalid(state: *mut c_void) -> c_int {
    API.get()
        .copied()
        .map_or(0, |api| unsafe { push_code(api, state, -20) })
}
