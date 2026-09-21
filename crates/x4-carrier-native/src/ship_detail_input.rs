use crate::abi_windows::LuaApi;
use crate::lua_table::{exact_keys, field_integer, field_string};
use core::ffi::c_void;
use observation_domain::{
    CaptureWindow, CargoObservation, CargoStorage, CargoWare, ObservationPolicyVersion,
    SectionRevisionId, ShipDetailDependency, ShipIdentity, ShipOwner, SourceEvidenceRef,
    detail_number, detail_outcome,
};

const KEYS: [&str; 16] = [
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
    "consistency",
    "consistency_reason",
    "wares_outcome",
    "storage_outcome",
    "wares",
    "storage",
];

pub unsafe fn cargo(
    api: LuaApi,
    state: *mut c_void,
    max_inner: usize,
) -> Option<(String, CargoObservation)> {
    if !unsafe { exact_keys(api, state, 2, &KEYS) } {
        return None;
    }
    let string = |key, limit| unsafe { field_string(api, state, 2, key, limit) };
    let source_scope = string("source_scope", 128)?;
    let dependency = unsafe { dependency(api, state) }?;
    let wares = unsafe {
        crate::lua_detail_array::read(api, state, "wares", max_inner, |index| {
            if !exact_keys(api, state, index, &["ware", "amount_items"]) {
                return None;
            }
            Some(CargoWare {
                ware: field_string(api, state, index, "ware", 128)?,
                amount_items: u64::try_from(field_integer(api, state, index, "amount_items")?)
                    .ok()?,
            })
        })
    }?;
    let storage = unsafe {
        crate::lua_detail_array::read(api, state, "storage", max_inner, |index| {
            if !exact_keys(
                api,
                state,
                index,
                &[
                    "transport",
                    "capacity_cubic_metres",
                    "occupied_cubic_metres",
                ],
            ) {
                return None;
            }
            Some(CargoStorage {
                transport: field_string(api, state, index, "transport", 128)?,
                capacity_cubic_metres: u32::try_from(field_integer(
                    api,
                    state,
                    index,
                    "capacity_cubic_metres",
                )?)
                .ok()?,
                occupied_cubic_metres: u32::try_from(field_integer(
                    api,
                    state,
                    index,
                    "occupied_cubic_metres",
                )?)
                .ok()?,
            })
        })
    }?;
    let record = CargoObservation {
        dependency,
        wares: detail_outcome(&string("wares_outcome", 32)?, wares).ok()?,
        storage: detail_outcome(&string("storage_outcome", 32)?, storage).ok()?,
    };
    record.validate(max_inner).ok()?;
    Some((source_scope, record))
}

pub(crate) unsafe fn dependency(api: LuaApi, state: *mut c_void) -> Option<ShipDetailDependency> {
    let string = |key, limit| unsafe { field_string(api, state, 2, key, limit) };
    Some(ShipDetailDependency {
        identity: ShipIdentity::new(string("identity", 20)?).ok()?,
        owner: ShipOwner::new(string("owner", 64)?).ok()?,
        core_revision: SectionRevisionId::new(detail_number(&string("core_revision", 20)?).ok()?)?,
        member_revision: SectionRevisionId::new(
            detail_number(&string("member_revision", 20)?).ok()?,
        )?,
        policy: ObservationPolicyVersion::new(
            u64::try_from(unsafe { field_integer(api, state, 2, "policy_version") }?).ok()?,
        )?,
        capture: CaptureWindow::new(
            detail_number(&string("capture_start_millis", 20)?).ok()?,
            detail_number(&string("capture_end_millis", 20)?).ok()?,
        )?,
        source: SourceEvidenceRef::new(string("source_evidence", 128)?)?,
        consistency: observation_domain::ShipRecordConsistency::from_fields(
            &string("consistency", 32)?,
            &string("consistency_reason", 32)?,
        )
        .ok()?,
    })
}
