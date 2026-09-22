use core::ffi::c_void;

use crate::abi_windows::LuaApi;
use crate::lua_table::{exact_keys, field_bool, field_integer, field_string};

const KEYS: [&str; 8] = [
    "profile",
    "faction_id",
    "discovery_revision",
    "disposition",
    "reason",
    "origin",
    "source_evidence",
    "mind_candidate",
];

pub unsafe fn record(api: LuaApi, state: *mut c_void) -> Option<crate::FactionCensusRecord> {
    if !unsafe { exact_keys(api, state, 2, &KEYS) } {
        return None;
    }
    Some(crate::FactionCensusRecord {
        faction_id: unsafe { field_string(api, state, 2, "faction_id", 128) }?,
        discovery_revision: u64::try_from(unsafe {
            field_integer(api, state, 2, "discovery_revision")
        }?)
        .ok()?,
        disposition: unsafe { field_string(api, state, 2, "disposition", 16) }?,
        reason: unsafe { field_string(api, state, 2, "reason", 48) }?,
        origin: unsafe { field_string(api, state, 2, "origin", 16) }?,
        source_evidence: unsafe { field_string(api, state, 2, "source_evidence", 128) }?,
        mind_candidate: unsafe { field_bool(api, state, 2, "mind_candidate") }?,
    })
}
