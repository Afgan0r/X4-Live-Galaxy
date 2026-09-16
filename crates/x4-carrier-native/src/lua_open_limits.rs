use crate::abi_windows::LuaApi;
use crate::lua_table::field_integer;
use crate::{ProducerAdmissionPolicy, ProducerLimits};
use core::ffi::c_void;

pub unsafe fn decode(
    api: LuaApi,
    state: *mut c_void,
) -> Option<(ProducerLimits, ProducerAdmissionPolicy)> {
    let data = unsafe { field_integer(api, state, 2, "data_message_bytes") }?;
    let control = unsafe { field_integer(api, state, 2, "control_message_bytes") }?;
    let records = unsafe { field_integer(api, state, 2, "max_records") }?;
    let content = unsafe { field_integer(api, state, 2, "max_content_bytes") }?;
    let canonical = unsafe { field_integer(api, state, 2, "max_canonical_bytes") }?;
    let batches = unsafe { field_integer(api, state, 2, "max_batches") }?;
    let work = unsafe { field_integer(api, state, 2, "max_work") }?;
    let age = unsafe { field_integer(api, state, 2, "max_age_millis") }?;
    let slots = unsafe { field_integer(api, state, 2, "pending_slots") }?;
    let attempts = unsafe { field_integer(api, state, 2, "max_attempts") }?;
    let retry = unsafe { field_integer(api, state, 2, "max_retry_age_millis") }?;
    let availability = unsafe { field_integer(api, state, 2, "availability_interval_millis") }?;
    if canonical != data
        || batches < records
        || age == 0
        || slots != 1
        || attempts == 0
        || attempts > 2
        || availability != 5000
    {
        return None;
    }
    let limits = ProducerLimits {
        data_message_bytes: data,
        control_message_bytes: control,
        max_records: records,
        max_raw_bytes: content,
        max_batches: batches,
        max_work: work,
        max_retry_age_millis: u64::try_from(retry).ok()?,
    };
    if !limits.valid() {
        return None;
    }
    Some((
        limits,
        ProducerAdmissionPolicy {
            max_inner_records: records,
            max_attempts: u8::try_from(attempts).ok()?,
            max_age_millis: u64::try_from(age).ok()?,
        },
    ))
}
pub unsafe fn inner(
    api: LuaApi,
    state: *mut c_void,
    heavy: Option<usize>,
    records: usize,
    policy: ProducerAdmissionPolicy,
) -> Option<usize> {
    if heavy == Some(1) {
        if policy.max_attempts != 1 {
            return None;
        }
        let inner = unsafe { field_integer(api, state, 2, "max_inner_records") }?;
        return (inner > 0 && inner <= records).then_some(inner);
    }
    if heavy.is_some() || policy.max_age_millis != 5000 || policy.max_attempts != 2 {
        return None;
    }
    Some(records)
}
