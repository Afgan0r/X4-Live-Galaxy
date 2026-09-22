use core::ffi::c_void;

use observation_domain::{
    CaptureWindow, SectionAvailability, SectionCoverage, SectionFreshness, SectionQuality,
    SectionState, SenderEvidence, ShipClass, ShipCoreRecord, ShipIdentity, ShipLocation, ShipOwner,
    ShipRecordConsistency, ShipType, SourceBoundary, SourceConsistency, SourceEpochStatus,
    SourceScopeId,
};

use crate::abi_windows::LuaApi;
use crate::lua_input::BeginInput;
use crate::lua_table::{exact_keys, field_bool, field_integer, field_string};
use crate::{ProducerSource, SectionEvidence};

const RECORD_KEYS: [&str; 9] = [
    "profile",
    "source_scope",
    "identity",
    "owner",
    "type",
    "class",
    "location",
    "consistency",
    "consistency_reason",
];

pub unsafe fn begin(
    api: LuaApi,
    state: *mut c_void,
    source: &ProducerSource,
    keys: &[&str],
) -> Option<BeginInput> {
    let section_key = unsafe { field_string(api, state, 2, "section_key", 128) }?;
    if !unsafe { exact_keys(api, state, 2, keys) }
        || unsafe { field_string(api, state, 2, "capture_clock", 32) }? != "game_time_millis"
        || unsafe { field_string(api, state, 2, "quality", 32) }? != "unknown"
        || unsafe { field_string(api, state, 2, "availability", 32) }? != "available"
        || unsafe { field_string(api, state, 2, "coverage", 32) }? != "partial"
        || unsafe { field_string(api, state, 2, "consistency", 32) }? != "observed_count_fill_only"
        || !unsafe { field_bool(api, state, 2, "stable_identity") }?
    {
        return None;
    }
    let expected_records = unsafe { field_integer(api, state, 2, "expected_records") }?;
    let start = decimal(&unsafe { field_string(api, state, 2, "capture_start_millis", 20) }?)?;
    let mut sender = SenderEvidence::legacy_default();
    sender.section_state = SectionState::with_evidence(
        CaptureWindow::new(start, start)?,
        SectionFreshness::Fresh,
        SectionQuality::Unknown,
        SectionAvailability::Available,
        SectionCoverage::Partial,
    );
    sender.source_consistency = SourceConsistency::ObservedCountFillOnly;
    sender.stable_identity = true;
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
            "transport_reconnect" => SourceBoundary::TransportReconnect,
            _ => return None,
        };
    let faction = observation_domain::ShipSectionIdentity::parse(&section_key)
        .and_then(|section| section.faction().map(str::to_owned));
    let source_scope = faction.map_or_else(
        || source.source_scope.clone(),
        |faction| format!("x4:faction:{faction}:ships"),
    );
    let evidence = SectionEvidence {
        source_scope,
        sender,
    };
    if section_key == "faction_census" {
        return Some(BeginInput::FactionCensus {
            evidence,
            expected_records,
        });
    }
    let _section = observation_domain::ShipSectionIdentity::parse(&section_key)?;
    Some(BeginInput::ShipCore {
        evidence,
        expected_records,
    })
}

pub unsafe fn is_ship_core(api: LuaApi, state: *mut c_void) -> bool {
    unsafe { field_string(api, state, 2, "profile", 32) }.as_deref() == Some("ship_core")
}

pub unsafe fn record(api: LuaApi, state: *mut c_void) -> Option<ShipCoreRecord> {
    if !unsafe { exact_keys(api, state, 2, &RECORD_KEYS) } {
        return None;
    }
    build_record(ShipCoreFields {
        source_scope: unsafe { field_string(api, state, 2, "source_scope", 128) }?,
        identity: unsafe { field_string(api, state, 2, "identity", 32) }?,
        owner: unsafe { field_string(api, state, 2, "owner", 64) }?,
        ship_type: unsafe { field_string(api, state, 2, "type", 128) }?,
        class: unsafe { field_string(api, state, 2, "class", 64) }?,
        location: unsafe { field_string(api, state, 2, "location", 128) }?,
        consistency: ShipRecordConsistency::from_fields(
            &unsafe { field_string(api, state, 2, "consistency", 32) }?,
            &unsafe { field_string(api, state, 2, "consistency_reason", 32) }?,
        )
        .ok()?,
    })
}

struct ShipCoreFields {
    source_scope: String,
    identity: String,
    owner: String,
    ship_type: String,
    class: String,
    location: String,
    consistency: ShipRecordConsistency,
}

fn build_record(fields: ShipCoreFields) -> Option<ShipCoreRecord> {
    Some(
        ShipCoreRecord::new(
            SourceScopeId::new(fields.source_scope)?,
            ShipIdentity::new(fields.identity).ok()?,
            ShipOwner::new(fields.owner).ok()?,
            ShipType::new(fields.ship_type).ok()?,
            ShipClass::new(fields.class).ok()?,
            ShipLocation::new(fields.location).ok()?,
            SenderEvidence::legacy_default(),
        )
        .with_consistency(fields.consistency),
    )
}

fn decimal(value: &str) -> Option<u64> {
    (!value.is_empty() && value.bytes().all(|byte| byte.is_ascii_digit()))
        .then(|| value.parse().ok())?
}

#[cfg(test)]
mod tests {
    use super::{ShipCoreFields, build_record};

    #[test]
    fn copied_fields_keep_a_lossless_identity_after_the_caller_changes() {
        let mut identity = "9007199254740993".to_owned();
        let record = build_record(ShipCoreFields {
            source_scope: "x4:faction:argon:ships".to_owned(),
            identity: identity.clone(),
            owner: "argon".to_owned(),
            ship_type: "ship_arg_l_destroyer_01_a_macro".to_owned(),
            class: "destroyer".to_owned(),
            location: "sector:argon_prime".to_owned(),
            consistency: observation_domain::ShipRecordConsistency::Consistent,
        })
        .expect("copied ship input is valid");
        identity.replace_range(.., "1");

        assert_eq!(record.identity().as_str(), "9007199254740993");
    }

    #[test]
    fn noncanonical_identity_is_rejected_before_producer_admission() {
        assert!(
            build_record(ShipCoreFields {
                source_scope: "x4:faction:argon:ships".to_owned(),
                identity: "9.007199254740993e15".to_owned(),
                owner: "argon".to_owned(),
                ship_type: "ship_arg_l_destroyer_01_a_macro".to_owned(),
                class: "destroyer".to_owned(),
                location: "sector:argon_prime".to_owned(),
                consistency: observation_domain::ShipRecordConsistency::Consistent,
            })
            .is_none()
        );
    }
}
