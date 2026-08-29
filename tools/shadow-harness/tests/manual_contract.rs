use mind_orchestration::{EvidenceClass, ProviderFailure, ProviderRequest, ShadowProvider};
use shadow_harness::{EvidenceRecord, SubscriptionAdapter};

#[test]
fn manual_adapter_uses_the_shared_port_and_is_not_quality_evidence() {
    fn accepts_provider<P: ShadowProvider>(_: &P) {}

    let adapter = SubscriptionAdapter::unavailable();
    accepts_provider(&adapter);
    assert_eq!(adapter.evidence(), EvidenceClass::ManualHarness);
    assert_eq!(adapter.preflight(), Err(ProviderFailure::Unavailable));
}

#[test]
fn evidence_is_redacted_and_bounded() {
    let record = EvidenceRecord::redacted("request-1", "codex", "model-1");
    assert!(record.is_redacted());
    assert!(record.is_bounded());
}

#[test]
fn no_implicit_subscription_process_is_exposed() {
    let _: fn(&mut SubscriptionAdapter, &ProviderRequest) -> Result<Vec<u8>, ProviderFailure> =
        ShadowProvider::propose;
}
