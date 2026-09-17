use core::ffi::c_void;

use observation_domain::{
    CaptureWindow, SectionAvailability, SectionCoverage, SectionFreshness, SectionQuality,
    SectionState, SenderEvidence, SourceBoundary, SourceConsistency, SourceEpochStatus,
};

use crate::abi_windows::LuaApi;
use crate::lua_table::{exact_keys, field_bool, field_integer, field_string};
use crate::{ProducerSource, SectionEvidence, SectionFinishEvidence, TypedFact};

pub enum BeginInput {
    Clock(SectionEvidence),
    ShipCore {
        evidence: SectionEvidence,
        expected_records: usize,
    },
}

pub enum RecordInput {
    Clock(TypedFact),
    ShipCore(observation_domain::ShipCoreRecord),
    ShipCargo(String, observation_domain::CargoObservation),
    ShipCrew(String, observation_domain::CrewObservation),
    ShipLoadout(String, observation_domain::LoadoutObservation),
}

const BEGIN_KEYS: [&str; 11] = [
    "section_key",
    "expected_records",
    "capture_start_millis",
    "capture_clock",
    "quality",
    "availability",
    "coverage",
    "consistency",
    "stable_identity",
    "source_epoch_status",
    "source_boundary",
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
) -> Option<BeginInput> {
    let section_key = unsafe { field_string(api, state, 2, "section_key", 64) }?;
    if section_key == "ship_core"
        || section_key.starts_with("ship_cargo:g")
        || section_key.starts_with("ship_crew:g")
        || section_key.starts_with("ship_loadout:g")
    {
        return unsafe { crate::lua_ship_input::begin(api, state, source, &BEGIN_KEYS) };
    }
    if !unsafe { exact_keys(api, state, 2, &BEGIN_KEYS) }
        || section_key != "carrier_b_realtime_sample"
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
    sender.source_epoch_status =
        match unsafe { field_string(api, state, 2, "source_epoch_status", 32) }?.as_str() {
            "unknown" => SourceEpochStatus::Unknown,
            "boundary_uncertain" => SourceEpochStatus::BoundaryUncertain,
            _ => return None,
        };
    sender.source_boundary =
        match unsafe { field_string(api, state, 2, "source_boundary", 32) }?.as_str() {
            "runtime_start" => SourceBoundary::RuntimeStart,
            "game_loaded" => SourceBoundary::GameLoaded,
            "lua_reload" => SourceBoundary::LuaReload,
            _ => return None,
        };
    sender.source_consistency = SourceConsistency::Unknown;
    sender.stable_identity = false;
    Some(BeginInput::Clock(SectionEvidence {
        source_scope: source.source_scope.clone(),
        sender,
    }))
}

pub unsafe fn record(api: LuaApi, state: *mut c_void) -> Option<RecordInput> {
    let profile = unsafe { field_string(api, state, 2, "profile", 32) };
    if matches!(
        profile.as_deref(),
        Some("ship_cargo" | "ship_crew" | "ship_loadout")
    ) {
        let bytes = crate::abi::PRODUCER
            .lock()
            .ok()?
            .as_ref()?
            .limits
            .data_message_bytes;
        if !unsafe { crate::lua_detail_budget::admit(api, state, bytes) } {
            return None;
        }
    }
    if unsafe { field_string(api, state, 2, "profile", 32) }.as_deref() == Some("ship_loadout") {
        let limit = crate::abi::PRODUCER.lock().ok()?.as_ref()?.inner_limit();
        return unsafe { crate::ship_loadout_input::loadout(api, state, limit) }
            .map(|(scope, record)| RecordInput::ShipLoadout(scope, record));
    }
    if unsafe { field_string(api, state, 2, "profile", 32) }.as_deref() == Some("ship_crew") {
        let limit = crate::abi::PRODUCER.lock().ok()?.as_ref()?.inner_limit();
        return unsafe { crate::ship_crew_input::crew(api, state, limit) }
            .map(|(scope, record)| RecordInput::ShipCrew(scope, record));
    }
    if unsafe { field_string(api, state, 2, "profile", 32) }.as_deref() == Some("ship_cargo") {
        let limit = crate::abi::PRODUCER.lock().ok()?.as_ref()?.inner_limit();
        return unsafe { crate::ship_detail_input::cargo(api, state, limit) }
            .map(|(scope, record)| RecordInput::ShipCargo(scope, record));
    }
    if unsafe { crate::lua_ship_input::is_ship_core(api, state) } {
        return unsafe { crate::lua_ship_input::record(api, state) }.map(RecordInput::ShipCore);
    }
    if !unsafe { exact_keys(api, state, 2, &FACT_KEYS) } {
        return None;
    }
    Some(RecordInput::Clock(TypedFact {
        entity_id: unsafe { field_string(api, state, 2, "entity_id", 128) }?,
        observation_version: u64::try_from(unsafe {
            field_integer(api, state, 2, "observation_version")
        }?)
        .ok()?,
        getter: unsafe { field_string(api, state, 2, "getter", 32) }?,
        raw_value: unsafe { field_string(api, state, 2, "raw_value", 96) }?,
        semantics: unsafe { field_string(api, state, 2, "semantics", 32) }?,
    }))
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
            "partial" => SectionCoverage::Partial,
            _ => return None,
        },
        consistency: match unsafe { field_string(api, state, 2, "consistency", 32) }?.as_str() {
            "unknown" => SourceConsistency::Unknown,
            "observed_count_fill_only" => SourceConsistency::ObservedCountFillOnly,
            _ => return None,
        },
        stable_identity: unsafe { field_bool(api, state, 2, "stable_identity") }?,
    })
}

fn decimal(value: &str) -> Option<u64> {
    (!value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| value.parse().ok())?
}
