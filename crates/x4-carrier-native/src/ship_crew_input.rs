use crate::abi_windows::LuaApi;
use crate::lua_table::{exact_keys, field_bool, field_integer, field_signed, field_string};
use core::ffi::c_void;
use observation_domain::{CrewObservation, CrewRole, CrewTier, FieldOutcome, detail_outcome};

const KEYS: [&str; 15] = [
    "profile",
    "source_scope",
    "identity",
    "owner",
    "core_revision",
    "member_revision",
    "policy_version",
    "capture_start_millis",
    "capture_end_millis",
    "source_evidence",
    "capacity_outcome",
    "capacity_people",
    "roles_outcome",
    "includepilot",
    "roles",
];

pub unsafe fn crew(
    api: LuaApi,
    state: *mut c_void,
    maximum: usize,
) -> Option<(String, CrewObservation)> {
    if !unsafe { exact_keys(api, state, 2, &KEYS) }
        || !unsafe { field_bool(api, state, 2, "includepilot") }?
    {
        return None;
    }
    let dependency = unsafe { crate::ship_detail_input::dependency(api, state) }?;
    let capacity_value =
        u32::try_from(unsafe { field_integer(api, state, 2, "capacity_people") }?).ok()?;
    let capacity = match unsafe { field_string(api, state, 2, "capacity_outcome", 32) }?.as_str() {
        "value" if capacity_value > 0 => FieldOutcome::Value(capacity_value),
        "zero" if capacity_value == 0 => FieldOutcome::Zero,
        _ => return None,
    };
    let roles = unsafe {
        crate::lua_detail_array::read(api, state, "roles", maximum, |index| {
            role(api, state, index, maximum)
        })
    }?;
    let record = CrewObservation {
        dependency,
        capacity,
        roles: detail_outcome(
            &unsafe { field_string(api, state, 2, "roles_outcome", 32) }?,
            roles,
        )
        .ok()?,
    };
    record.validate(maximum).ok()?;
    Some((
        unsafe { field_string(api, state, 2, "source_scope", 128) }?,
        record,
    ))
}

unsafe fn role(api: LuaApi, state: *mut c_void, index: i32, maximum: usize) -> Option<CrewRole> {
    if !unsafe {
        exact_keys(
            api,
            state,
            index,
            &[
                "id",
                "amount_people",
                "reported_numtiers",
                "canhire",
                "tiers",
            ],
        )
    } {
        return None;
    }
    // Nested array reader takes its owner index explicitly; no retained Lua pointers.
    let tiers = unsafe {
        crate::lua_detail_array::read_at(api, state, index, "tiers", maximum, |i| {
            if !exact_keys(
                api,
                state,
                i,
                &["name", "skill_lower_threshold", "amount_people"],
            ) {
                return None;
            }
            Some(CrewTier {
                name: field_string(api, state, i, "name", 128)?,
                skill_lower_threshold: field_signed(api, state, i, "skill_lower_threshold")?,
                amount_people: u32::try_from(field_integer(api, state, i, "amount_people")?)
                    .ok()?,
            })
        })
    }?;
    Some(CrewRole {
        id: unsafe { field_string(api, state, index, "id", 128) }?,
        amount_people: u32::try_from(unsafe { field_integer(api, state, index, "amount_people") }?)
            .ok()?,
        reported_numtiers: u32::try_from(unsafe {
            field_integer(api, state, index, "reported_numtiers")
        }?)
        .ok()?,
        canhire: unsafe { field_bool(api, state, index, "canhire") }?,
        tiers,
    })
}
