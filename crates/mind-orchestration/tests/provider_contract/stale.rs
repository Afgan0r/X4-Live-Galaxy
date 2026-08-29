use super::*;
use mind_domain::{AdmissionDecision, AdmissionRejection, MindAggregate};
use mind_orchestration::{DeliberationRunner, RunnerOutcome};
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
            return;
        };
        assert_eq!(
            admission,
            AdmissionDecision::Rejected(AdmissionRejection::CurrentState)
        );
        assert!(admission.pending_commit(&prior).is_none());
    }
}
