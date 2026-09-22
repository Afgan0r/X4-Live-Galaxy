#[test]
fn full_set_fixture_uses_production_idle_recovery() {
    let fixture = include_str!("../examples/carrier_b_ship_local/full_set_peer.rs");
    assert!(fixture.contains("recover_idle("));
    assert!(!fixture.contains(".skip_stale_ship_scope()"));
}
