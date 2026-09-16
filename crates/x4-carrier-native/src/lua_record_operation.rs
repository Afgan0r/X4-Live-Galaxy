use crate::abi::{API, push_code};
use crate::lua_input::RecordInput;
use crate::lua_producer_context::{context, with_producer};
use crate::lua_progress::error_code;
use core::ffi::{c_int, c_void};
pub unsafe extern "C" fn push_record(state: *mut c_void) -> c_int {
    let Some((api, handle)) = (unsafe { context(state) }) else {
        return API
            .get()
            .copied()
            .map_or(0, |api| unsafe { push_code(api, state, -20) });
    };
    let Some(input) = (unsafe { crate::lua_input::record(api, state) }) else {
        return unsafe { push_code(api, state, -20) };
    };
    let result = with_producer(handle, |producer| match input {
        RecordInput::Clock(fact) => producer.push_record(&fact),
        RecordInput::ShipCore(record) => producer.push_ship_core(&record),
        RecordInput::ShipCargo(scope, record) => producer.push_ship_cargo(&scope, &record),
        RecordInput::ShipCrew(scope, record) => producer.push_ship_crew(&scope, &record),
        RecordInput::ShipLoadout(scope, record) => producer.push_ship_loadout(&scope, &record),
    });
    unsafe { push_code(api, state, result.map_or_else(error_code, |()| 0)) }
}
