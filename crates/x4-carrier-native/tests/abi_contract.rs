use x4_carrier_native::{ABI_VERSION, contained_status, module_contract};

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

    assert_eq!(contract.abi_version, 1, "the ABI version must be frozen at one");
    assert_eq!(contract.operations, EXPECTED_OPERATIONS);
    assert_eq!(ABI_VERSION, contract.abi_version);
}

#[test]
fn panic_is_contained_at_the_c_entry_boundary() {
    assert_eq!(contained_status(|| panic!("injected test panic")), -1);
}
