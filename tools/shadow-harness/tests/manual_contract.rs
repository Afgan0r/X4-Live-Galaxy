use mind_orchestration::{EvidenceClass, ProviderFailure, ProviderRequest, ShadowProvider};
use shadow_harness::{EvidenceRecord, SubscriptionAdapter};

type ProviderMethod = fn(&mut SubscriptionAdapter, &ProviderRequest) -> Result<Vec<u8>, ProviderFailure>;

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
fn manifest_pins_every_sd_case_and_manual_evidence_class() {
    let manifest = include_str!("../../../shadow-deliberation-evals/v1/manifest.json");
    assert!(EvidenceRecord::validates_manifest(manifest));
}

#[test]
fn no_implicit_subscription_process_is_exposed() {
    let _: ProviderMethod = ShadowProvider::propose;
}
