#[test]
fn full_set_fixture_uses_production_idle_recovery() {
    let fixture = include_str!("../examples/carrier_b_ship_local/full_set_peer.rs");
    let recovery = include_str!("../examples/carrier_b_ship_local/full_set_recovery.rs");
    assert!(fixture.contains("full_set_recovery::recover("));
    assert!(recovery.contains("recover_idle("));
    assert!(!fixture.contains(".skip_stale_ship_scope()"));
}
