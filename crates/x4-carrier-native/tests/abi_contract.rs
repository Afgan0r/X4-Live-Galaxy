use x4_carrier_native::{
    ABI_VERSION, InitializerError, OPERATION_UNAVAILABLE, contained_status,
    luaopen_live_galaxy_carrier, module_contract, require_lua_symbols,
};

const EXPECTED_OPERATIONS: [&str; 7] = [
    "abi_version",
    "open",
    "connection",
    "try_send",
    "poll",
    "reset",
    "close",
];

#[test]
fn initializer_contract_exposes_the_versioned_transport_only_api() {
    let contract = module_contract();

    assert_eq!(
        contract.abi_version, 1,
        "the ABI version must be frozen at one"
    );
    assert_eq!(contract.operations, EXPECTED_OPERATIONS);
    assert_eq!(ABI_VERSION, contract.abi_version);
}

#[test]
fn panic_is_contained_at_the_c_entry_boundary() {
    assert_eq!(contained_status(|| panic!("injected test panic")), -1);
}

#[test]
fn initializer_failure_is_distinct_from_operation_unavailable() {
    let missing = require_lua_symbols(|name| name != "lua_setfield");

    assert_eq!(
        missing,
        Err(InitializerError::MissingHostSymbol("lua_setfield"))
    );
    assert_ne!(OPERATION_UNAVAILABLE, 0);
    assert_eq!(luaopen_live_galaxy_carrier(core::ptr::null_mut()), 0);
}
