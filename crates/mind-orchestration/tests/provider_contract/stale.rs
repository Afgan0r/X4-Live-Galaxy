use super::*;
use mind_domain::{AdmissionDecision, AdmissionRejection, MindAggregate};
use mind_orchestration::{DeliberationRunner, RunnerOutcome};
use std::sync::{
    Arc,
    atomic::{AtomicUsize, Ordering},
};
#[test]
fn stale_provider_and_cache_reject_before_pending_commit() {
    let (Ok(request), Ok(candidate)) = (request(), candidate()) else {
        return;
    };
    let canonical = canonical("stale", request);
    assert!(canonical.is_ok());
    let Ok(canonical) = canonical else { return };
    let prior = MindAggregate::empty(Faction::Zya);
    let mut runner = DeliberationRunner::new();
    let mut scheduler = scheduled();
    let mut provider = FakeProvider {
        outcome: Ok(candidate.clone()),
    };
    let provider = runner.run(
        &mut provider,
        &canonical,
        &prior,
        RunContext {
            current_snapshot_identity: "snapshot-zya-2",
            scheduler: &mut scheduler,
            faction: Faction::Zya,
        },
    );
    let cached = runner.run_cached(
        &canonical,
        &prior,
        &candidate,
        EvidenceClass::DeterministicFixture,
        RunContext {
            current_snapshot_identity: "snapshot-zya-2",
            scheduler: &mut scheduler,
            faction: Faction::Zya,
        },
    );
    for outcome in [provider, cached] {
        let RunnerOutcome::Admitted { admission, .. } = outcome else {
            panic!("stale provider/cache work must produce an admission outcome");
        };
        assert_eq!(
            admission,
            AdmissionDecision::Rejected(AdmissionRejection::CurrentState)
        );
        assert!(admission.pending_commit(&prior).is_none());
    }
}

struct CountingProvider {
    calls: Arc<AtomicUsize>,
    candidate: Vec<u8>,
}

impl ShadowProvider for CountingProvider {
    fn propose(&mut self, _: &ProviderRequest) -> Result<Vec<u8>, ProviderFailure> {
        self.calls.fetch_add(1, Ordering::Relaxed);
        Ok(self.candidate.clone())
    }

    fn evidence(&self) -> EvidenceClass {
        EvidenceClass::DeterministicFixture
    }
}

#[test]
fn stale_provider_request_is_rejected_before_provider_invocation() {
    let (Ok(request), Ok(candidate)) = (request(), candidate()) else {
        panic!("canonical stale-provider fixture must be constructible");
    };
    let canonical = canonical("stale-before-provider", request);
    assert!(canonical.is_ok());
    let Ok(canonical) = canonical else { return };
    let calls = Arc::new(AtomicUsize::new(0));
    let mut provider = CountingProvider {
        calls: Arc::clone(&calls),
        candidate,
    };
    let prior = MindAggregate::empty(Faction::Zya);
    let mut runner = DeliberationRunner::new();
    let mut scheduler = scheduled();
    let outcome = runner.run(
        &mut provider,
        &canonical,
        &prior,
        RunContext {
            current_snapshot_identity: "snapshot-zya-2",
            scheduler: &mut scheduler,
            faction: Faction::Zya,
        },
    );
    assert!(matches!(
        outcome,
        RunnerOutcome::Admitted {
            admission: AdmissionDecision::Rejected(AdmissionRejection::CurrentState),
            ..
        }
    ));
    assert_eq!(calls.load(Ordering::Relaxed), 0);
    assert_eq!(prior, MindAggregate::empty(Faction::Zya));
}
