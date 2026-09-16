use core::ffi::c_void;

use observation_domain::{SourceBoundary, SourceEpochStatus};

use crate::abi_windows::LuaApi;
use crate::lua_table::{exact_keys, field_integer, field_string};
use crate::{ProducerAdmissionPolicy, ProducerLimits};

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
    pub policy: ProducerAdmissionPolicy,
    pub limits: ProducerLimits,
    pub source_scope: String,
    pub epoch_status: SourceEpochStatus,
    pub boundary: SourceBoundary,
}

pub unsafe fn decode(api: LuaApi, state: *mut c_void) -> Option<OpenArgs> {
    let heavy = unsafe { field_integer(api, state, 2, "heavy_profile_version") };
    let mut keys = LIMIT_KEYS.to_vec();
    if heavy.is_some() {
        keys.extend(["heavy_profile_version", "max_inner_records"]);
    }
    if unsafe { crate::abi::integer(api, state, 1) } != Some(2)
        || !unsafe { exact_keys(api, state, 2, &keys) }
        || !unsafe { exact_keys(api, state, 3, &SOURCE_KEYS) }
    {
        return None;
    }
    let (limits, mut policy) = unsafe { crate::lua_open_limits::decode(api, state) }?;
    policy.max_inner_records =
        unsafe { crate::lua_open_limits::inner(api, state, heavy, limits.max_records, policy) }?;
    let (source_scope, epoch_status, boundary) = unsafe { decode_source(api, state) }?;
    let _validated_scope = observation_domain::SourceScopeId::new(source_scope.clone())?;
    Some(OpenArgs {
        policy,
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
