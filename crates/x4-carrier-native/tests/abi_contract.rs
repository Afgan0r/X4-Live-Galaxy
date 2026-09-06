use std::num::NonZeroUsize;
use x4_carrier_native::{
    ABI_VERSION, CarrierError, CarrierLimits, CloseOutcome, ControlPollOutcome, HandleRegistry,
    InitializerError, OPERATION_UNAVAILABLE, OpenConfig, SendOutcome, contained_status,
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

fn limits() -> CarrierLimits {
    CarrierLimits::new(NonZeroUsize::new(8).unwrap(), NonZeroUsize::new(4).unwrap())
}

#[test]
fn accepted_bytes_are_owned_and_capacity_is_one_complete_message() {
    let mut registry = HandleRegistry::new();
    let handle = registry.open(OpenConfig::current(limits())).unwrap();
    let mut caller = b"12345678".to_vec();

    assert_eq!(
        registry.try_send(handle, &caller),
        SendOutcome::LocalHandoff
    );
    caller.fill(b'x');
    assert_eq!(registry.pending_copy(handle), Some(b"12345678".to_vec()));
    assert_eq!(
        registry.try_send(handle, b"next"),
        SendOutcome::CapacityUnavailable
    );
    assert_eq!(
        registry.try_send(handle, b"123456789"),
        SendOutcome::Rejected(CarrierError::MessageTooLarge)
    );
}

#[test]
fn versions_generations_and_close_are_checked_without_aliasing() {
    let mut registry = HandleRegistry::new();
    let wrong = OpenConfig {
        abi_version: 2,
        ..OpenConfig::current(limits())
    };
    assert_eq!(registry.open(wrong), Err(CarrierError::WrongVersion));

    let first = registry.open(OpenConfig::current(limits())).unwrap();
    assert_eq!(registry.close(first), CloseOutcome::Closed);
    assert_eq!(registry.close(first), CloseOutcome::AlreadyClosed);
    let second = registry.open(OpenConfig::current(limits())).unwrap();
    assert_ne!(first, second);
    assert_eq!(
        registry.try_send(first, b"stale"),
        SendOutcome::Rejected(CarrierError::StaleHandle)
    );
}

#[test]
fn short_control_output_does_not_consume_the_message() {
    let mut registry = HandleRegistry::new();
    let handle = registry.open(OpenConfig::current(limits())).unwrap();
    registry.queue_control(handle, b"ctrl").unwrap();

    assert_eq!(
        registry.poll_control(handle, 3),
        ControlPollOutcome::Rejected(CarrierError::InvalidOutputCapacity)
    );
    assert_eq!(
        registry.poll_control(handle, 4),
        ControlPollOutcome::Message(b"ctrl".to_vec())
    );
    assert_eq!(
        registry.poll_control(handle, 4),
        ControlPollOutcome::NoMessage
    );
}

#[test]
fn generation_exhaustion_rejects_instead_of_wrapping() {
    let mut registry = HandleRegistry::starting_at(u32::MAX);
    let config = OpenConfig::current(limits());
    let _last = registry.open(config).unwrap();
    assert_eq!(
        registry.open(config),
        Err(CarrierError::GenerationExhausted)
    );
}
