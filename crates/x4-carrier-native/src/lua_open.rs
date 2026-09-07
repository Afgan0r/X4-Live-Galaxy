use core::ffi::c_void;

use observation_domain::{SourceBoundary, SourceEpochStatus};

use crate::ProducerLimits;
use crate::abi_windows::LuaApi;
use crate::lua_table::{exact_keys, field_integer, field_string};

const LIMIT_KEYS: [&str; 12] = [
    "data_message_bytes",
    "control_message_bytes",
    "max_records",
    "max_content_bytes",
    "max_canonical_bytes",
    "max_batches",
    "max_work",
    "max_age_millis",
    "pending_slots",
    "max_attempts",
    "max_retry_age_millis",
    "availability_interval_millis",
];
const SOURCE_KEYS: [&str; 3] = ["source_scope", "source_epoch_status", "source_boundary"];

pub struct OpenArgs {
    pub limits: ProducerLimits,
    pub source_scope: String,
    pub epoch_status: SourceEpochStatus,
    pub boundary: SourceBoundary,
}

pub unsafe fn decode(api: LuaApi, state: *mut c_void) -> Option<OpenArgs> {
    if unsafe { crate::abi::integer(api, state, 1) } != Some(2)
        || !unsafe { exact_keys(api, state, 2, &LIMIT_KEYS) }
        || !unsafe { exact_keys(api, state, 3, &SOURCE_KEYS) }
    {
        return None;
    }
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
    if data != 2_048
        || control != 512
        || records != 1
        || content != 96
        || canonical != 2_048
        || batches != 1
        || work != 1
        || age != 5_000
        || slots != 1
        || attempts != 2
        || availability != 5_000
    {
        return None;
    }
    let retry = u64::try_from(retry).ok()?;
    let limits = ProducerLimits {
        data_message_bytes: data,
        control_message_bytes: control,
        max_records: records,
        max_raw_bytes: content,
        max_batches: batches,
        max_work: work,
        max_retry_age_millis: retry,
    };
    if !limits.valid() {
        return None;
    }
    let (source_scope, epoch_status, boundary) = unsafe { decode_source(api, state) }?;
    if source_scope != "x4:carrier_b_acceptance" {
        return None;
    }
    Some(OpenArgs {
        limits,
        source_scope,
        epoch_status,
        boundary,
    })
}

unsafe fn decode_source(
    api: LuaApi,
    state: *mut c_void,
) -> Option<(String, SourceEpochStatus, SourceBoundary)> {
    let scope = unsafe { field_string(api, state, 3, "source_scope", 128) }?;
    let epoch = match unsafe { field_string(api, state, 3, "source_epoch_status", 32) }?.as_str() {
        "unknown" => SourceEpochStatus::Unknown,
        "boundary_uncertain" => SourceEpochStatus::BoundaryUncertain,
        _ => return None,
    };
    let boundary = match unsafe { field_string(api, state, 3, "source_boundary", 32) }?.as_str() {
        "runtime_start" => SourceBoundary::RuntimeStart,
        "game_loaded" => SourceBoundary::GameLoaded,
        "lua_reload" => SourceBoundary::LuaReload,
        "transport_reconnect" => SourceBoundary::TransportReconnect,
        _ => return None,
    };
    Some((scope, epoch, boundary))
}
