use mind_orchestration::{EvidenceClass, ProviderFailure, ProviderRequest, ShadowProvider};
use shadow_harness::{
    BenchmarkProcess, CodexProcess, EvidenceRecord, SubscriptionAdapter, run_cli, validate_corpus,
};
use std::{
    path::Path,
    sync::{
        Arc,
        atomic::{AtomicUsize, Ordering},
    },
};

type ProviderMethod = fn(
    &mut SubscriptionAdapter<CodexProcess>,
    &ProviderRequest,
) -> Result<Vec<u8>, ProviderFailure>;

#[test]
fn manual_adapter_uses_the_shared_port_and_is_not_quality_evidence() {
    fn accepts_provider<P: ShadowProvider>(_: &P) {}

    let adapter = SubscriptionAdapter::<CodexProcess>::unavailable();
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
    assert!(validate_corpus(Path::new(
        "../../shadow-deliberation-evals/v1"
    )));
}

#[test]
fn no_implicit_subscription_process_is_exposed() {
    let _: ProviderMethod = ShadowProvider::propose;
}

struct FakeProcess {
    calls: Arc<AtomicUsize>,
}
impl BenchmarkProcess for FakeProcess {
    fn invoke(&mut self, identity: &str) -> Result<Vec<u8>, ProviderFailure> {
        let _ = identity;
        self.calls.fetch_add(1, Ordering::Relaxed);
        Ok(br#"{"schema_version":"schema-v1","capability":"DefenseAndMilitaryStrategy","priority":1,"horizon":"short","supporting_fact_ids":["ZYA:military:fleet"],"trade_offs":["preserve logistics"],"explanation":"hold frontier","command_id":"mind-zya-shadow-1"}"#.to_vec())
    }
}

#[test]
fn explicit_cli_loads_the_owned_corpus_and_invokes_the_process() {
    let corpus = std::fs::canonicalize("../../shadow-deliberation-evals/v1");
    assert!(corpus.is_ok());
    let Ok(corpus) = corpus else {
        return;
    };
    let args = vec![
        "--benchmark".into(),
        "--corpus".into(),
        corpus.display().to_string(),
    ];
    let calls = Arc::new(AtomicUsize::new(0));
    let result = run_cli(
        &args,
        FakeProcess {
            calls: Arc::clone(&calls),
        },
    );
    assert!(result.is_ok());
    let Ok(records) = result else {
        return;
    };
    assert_eq!(records.len(), 1);
    assert_eq!(calls.load(Ordering::Relaxed), 1);
    assert!(records.iter().all(EvidenceRecord::is_redacted));
}

#[test]
fn explicit_cli_fails_for_provider_or_malformed_candidate() {
    let corpus = std::fs::canonicalize("../../shadow-deliberation-evals/v1");
    assert!(corpus.is_ok());
    let Ok(corpus) = corpus else {
        return;
    };
    let args = vec![
        "--benchmark".into(),
        "--corpus".into(),
        corpus.display().to_string(),
    ];
    let calls = Arc::new(AtomicUsize::new(0));
    struct Failed;
    impl BenchmarkProcess for Failed {
        fn invoke(&mut self, _: &str) -> Result<Vec<u8>, ProviderFailure> {
            Err(ProviderFailure::Timeout)
        }
    }
    assert!(run_cli(&args, Failed).is_err());
    struct Malformed;
    impl BenchmarkProcess for Malformed {
        fn invoke(&mut self, _: &str) -> Result<Vec<u8>, ProviderFailure> {
            Ok(b"{}".to_vec())
        }
    }
    assert!(run_cli(&args, Malformed).is_err());
    assert_eq!(calls.load(Ordering::Relaxed), 0);
}
