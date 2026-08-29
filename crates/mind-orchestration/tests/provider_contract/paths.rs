use super::*;
use mind_domain::{MindAggregate, RequestEligibility};
use mind_orchestration::{DeliberationRunner, RunnerOutcome};
#[test]
fn scheduler_terminal_paths_clear_or_pause_then_reconcile() {
    let (Ok(request), Ok(candidate)) = (request(), candidate()) else {
        return;
    };
    let canonical = canonical("request-zya", request);
    assert!(canonical.is_ok());
    let Ok(canonical) = canonical else { return };
    let prior = MindAggregate::empty(Faction::Zya);
    let mut runner = DeliberationRunner::new();
    let mut scheduler = scheduled();
    let mut provider = FakeProvider {
        outcome: Ok(candidate.clone()),
    };
    let accepted = runner.run(&mut provider, &canonical, &prior, context(&mut scheduler));
    assert!(matches!(accepted, RunnerOutcome::Admitted { .. }));
    assert_eq!(scheduler.outstanding_count(Faction::Zya), 0);
    let _ = scheduler.eligibility(Faction::Zya, FactionTrigger::StrategicTick(3));
    let cached = runner.run_cached(
        &canonical,
        &prior,
        &candidate,
        EvidenceClass::DeterministicFixture,
        context(&mut scheduler),
    );
    assert!(matches!(cached, RunnerOutcome::Admitted { .. }));
    assert_eq!(scheduler.outstanding_count(Faction::Zya), 0);
    let _ = scheduler.eligibility(Faction::Zya, FactionTrigger::StrategicTick(5));
    let mut failed = FakeProvider {
        outcome: Err(ProviderFailure::Timeout),
    };
    assert!(matches!(
        runner.run(&mut failed, &canonical, &prior, context(&mut scheduler)),
        RunnerOutcome::Degraded(_)
    ));
    assert_eq!(
        scheduler.eligibility(Faction::Zya, FactionTrigger::StrategicTick(6)),
        RequestEligibility::PausedAwaitingReconciliation
    );
    assert_eq!(
        scheduler.reconcile(Faction::Zya, 7),
        RequestEligibility::Reconciled
    );
    assert!(matches!(
        scheduler.eligibility(Faction::Zya, FactionTrigger::StrategicTick(9)),
        RequestEligibility::Eligible(_)
    ));
}
