use crate::abi_windows::LuaApi;
use crate::lua_table::{exact_keys, field_bool, field_integer, field_signed, field_string};
use core::ffi::c_void;
use observation_domain::{
    InstalledSlot, InstalledSoftware, LoadoutObservation, MissileCargo, ShipUnit, VirtualSlot,
    detail_outcome,
};
const KEYS: [&str; 23] = [
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
    "physical_outcome",
    "virtual_outcome",
    "software_outcome",
    "missiles_outcome",
    "units_outcome",
    "physical",
    "virtual_slots",
    "software",
    "missiles",
    "units",
    "onlydrones",
];
pub unsafe fn loadout(
    api: LuaApi,
    state: *mut c_void,
    maximum: usize,
) -> Option<(String, LoadoutObservation)> {
    if !unsafe { exact_keys(api, state, 2, &KEYS) }
        || unsafe { field_bool(api, state, 2, "onlydrones") }?
    {
        return None;
    }
    let dependency = unsafe { crate::ship_detail_input::dependency(api, state) }?;
    let physical = unsafe {
        crate::lua_detail_array::read(api, state, "physical", maximum, |i| physical(api, state, i))
    }?;
    let virtual_slots = unsafe {
        crate::lua_detail_array::read(api, state, "virtual_slots", maximum, |i| {
            virtual_slot(api, state, i)
        })
    }?;
    let software = unsafe {
        crate::lua_detail_array::read(api, state, "software", maximum, |i| software(api, state, i))
    }?;
    let missiles = unsafe {
        crate::lua_detail_array::read(api, state, "missiles", maximum, |i| missile(api, state, i))
    }?;
    let units = unsafe {
        crate::lua_detail_array::read(api, state, "units", maximum, |i| unit(api, state, i))
    }?;
    let outcome = |key| unsafe { field_string(api, state, 2, key, 32) };
    let record = LoadoutObservation {
        dependency,
        physical: detail_outcome(&outcome("physical_outcome")?, physical).ok()?,
        virtual_slots: detail_outcome(&outcome("virtual_outcome")?, virtual_slots).ok()?,
        software: detail_outcome(&outcome("software_outcome")?, software).ok()?,
        missiles: detail_outcome(&outcome("missiles_outcome")?, missiles).ok()?,
        units: detail_outcome(&outcome("units_outcome")?, units).ok()?,
    };
    record.validate(maximum).ok()?;
    Some((
        unsafe { field_string(api, state, 2, "source_scope", 128) }?,
        record,
    ))
}
unsafe fn physical(api: LuaApi, state: *mut c_void, i: i32) -> Option<InstalledSlot> {
    if !unsafe {
        exact_keys(
            api,
            state,
            i,
            &["kind", "slot", "component", "macro_name", "path", "group"],
        )
    } {
        return None;
    }
    Some(InstalledSlot {
        kind: unsafe { field_string(api, state, i, "kind", 32) }?,
        slot: u32::try_from(unsafe { field_integer(api, state, i, "slot") }?).ok()?,
        component: unsafe { field_string(api, state, i, "component", 20) }?,
        macro_name: unsafe { field_string(api, state, i, "macro_name", 128) }?,
        path: unsafe { field_string(api, state, i, "path", 128) }?,
        group: unsafe { field_string(api, state, i, "group", 128) }?,
    })
}
unsafe fn virtual_slot(api: LuaApi, state: *mut c_void, i: i32) -> Option<VirtualSlot> {
    if !unsafe { exact_keys(api, state, i, &["kind", "slot", "macro_name"]) } {
        return None;
    }
    Some(VirtualSlot {
        kind: unsafe { field_string(api, state, i, "kind", 32) }?,
        slot: u32::try_from(unsafe { field_integer(api, state, i, "slot") }?).ok()?,
        macro_name: unsafe { field_string(api, state, i, "macro_name", 128) }?,
    })
}
unsafe fn software(api: LuaApi, state: *mut c_void, i: i32) -> Option<InstalledSoftware> {
    if !unsafe { exact_keys(api, state, i, &["maximum", "current"]) } {
        return None;
    }
    Some(InstalledSoftware {
        maximum: unsafe { field_string(api, state, i, "maximum", 128) }?,
        current: unsafe { field_string(api, state, i, "current", 128) }?,
    })
}
unsafe fn missile(api: LuaApi, state: *mut c_void, i: i32) -> Option<MissileCargo> {
    if !unsafe { exact_keys(api, state, i, &["ware", "macro_name", "amount_raw"]) } {
        return None;
    }
    Some(MissileCargo {
        ware: unsafe { field_string(api, state, i, "ware", 128) }?,
        macro_name: unsafe { field_string(api, state, i, "macro_name", 128) }?,
        amount_raw: unsafe { field_signed(api, state, i, "amount_raw") }?,
    })
}
unsafe fn unit(api: LuaApi, state: *mut c_void, i: i32) -> Option<ShipUnit> {
    if !unsafe { exact_keys(api, state, i, &["macro_name", "category", "amount_items"]) } {
        return None;
    }
    Some(ShipUnit {
        macro_name: unsafe { field_string(api, state, i, "macro_name", 128) }?,
        category: unsafe { field_string(api, state, i, "category", 128) }?,
        amount_items: u32::try_from(unsafe { field_integer(api, state, i, "amount_items") }?)
            .ok()?,
    })
}
