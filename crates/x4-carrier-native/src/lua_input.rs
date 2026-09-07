use core::ffi::c_void;

use observation_domain::{
    CaptureWindow, SectionAvailability, SectionCoverage, SectionFreshness, SectionQuality,
    SectionState, SenderEvidence, SourceConsistency,
};

use crate::abi_windows::LuaApi;
use crate::lua_table::{exact_keys, field_bool, field_integer, field_string};
use crate::{ProducerSource, SectionEvidence, SectionFinishEvidence, TypedFact};

const BEGIN_KEYS: [&str; 9] = [
    "section_key",
    "expected_records",
    "capture_start_millis",
    "capture_clock",
    "quality",
    "availability",
    "coverage",
    "consistency",
    "stable_identity",
];
const FACT_KEYS: [&str; 5] = [
    "entity_id",
    "observation_version",
    "getter",
    "raw_value",
    "semantics",
];
const FINISH_KEYS: [&str; 7] = [
    "capture_end_millis",
    "success",
    "quality",
    "availability",
    "coverage",
    "consistency",
    "stable_identity",
];

pub unsafe fn begin(
    api: LuaApi,
    state: *mut c_void,
    source: &ProducerSource,
) -> Option<SectionEvidence> {
    if !unsafe { exact_keys(api, state, 2, &BEGIN_KEYS) }
        || unsafe { field_string(api, state, 2, "section_key", 64) }? != "carrier_b_realtime_sample"
        || unsafe { field_integer(api, state, 2, "expected_records") }? != 1
        || unsafe { field_string(api, state, 2, "capture_clock", 32) }? != "game_time_millis"
        || unsafe { field_string(api, state, 2, "quality", 32) }? != "unknown"
        || unsafe { field_string(api, state, 2, "availability", 32) }? != "available"
        || unsafe { field_string(api, state, 2, "coverage", 32) }? != "point_measurement"
        || unsafe { field_string(api, state, 2, "consistency", 32) }? != "unknown"
        || unsafe { field_bool(api, state, 2, "stable_identity") }?
    {
        return None;
    }
    let start = decimal(&unsafe { field_string(api, state, 2, "capture_start_millis", 20) }?)?;
    let mut sender = SenderEvidence::legacy_default();
    sender.section_state = SectionState::with_evidence(
        CaptureWindow::new(start, start)?,
        SectionFreshness::Fresh,
        SectionQuality::Unknown,
        SectionAvailability::Available,
        SectionCoverage::PointMeasurement,
    );
    sender.source_epoch_status = source.source_epoch_status;
    sender.source_boundary = source.source_boundary;
    sender.source_consistency = SourceConsistency::Unknown;
    sender.stable_identity = false;
    Some(SectionEvidence {
        source_scope: source.source_scope.clone(),
        sender,
    })
}

pub unsafe fn fact(api: LuaApi, state: *mut c_void) -> Option<TypedFact> {
    if !unsafe { exact_keys(api, state, 2, &FACT_KEYS) } {
        return None;
    }
    Some(TypedFact {
        entity_id: unsafe { field_string(api, state, 2, "entity_id", 128) }?,
        observation_version: u64::try_from(unsafe {
            field_integer(api, state, 2, "observation_version")
        }?)
        .ok()?,
        getter: unsafe { field_string(api, state, 2, "getter", 32) }?,
        raw_value: unsafe { field_string(api, state, 2, "raw_value", 96) }?,
        semantics: unsafe { field_string(api, state, 2, "semantics", 32) }?,
    })
}

pub unsafe fn finish(api: LuaApi, state: *mut c_void) -> Option<SectionFinishEvidence> {
    if !unsafe { exact_keys(api, state, 2, &FINISH_KEYS) } {
        return None;
    }
    let end = decimal(&unsafe { field_string(api, state, 2, "capture_end_millis", 20) }?)?;
    Some(SectionFinishEvidence {
        capture_end_millis: end,
        succeeded: unsafe { field_bool(api, state, 2, "success") }?,
        quality: match unsafe { field_string(api, state, 2, "quality", 32) }?.as_str() {
            "unknown" => SectionQuality::Unknown,
            _ => return None,
        },
        availability: match unsafe { field_string(api, state, 2, "availability", 32) }?.as_str() {
            "available" => SectionAvailability::Available,
            _ => return None,
        },
        coverage: match unsafe { field_string(api, state, 2, "coverage", 32) }?.as_str() {
            "point_measurement" => SectionCoverage::PointMeasurement,
            _ => return None,
        },
        consistency: match unsafe { field_string(api, state, 2, "consistency", 32) }?.as_str() {
            "unknown" => SourceConsistency::Unknown,
            _ => return None,
        },
        stable_identity: unsafe { field_bool(api, state, 2, "stable_identity") }?,
    })
}

fn decimal(value: &str) -> Option<u64> {
    (!value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| value.parse().ok())?
}
