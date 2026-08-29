use crate::benchmark_fixture::BenchmarkFixture;
use crate::{BenchmarkProcess, EvidenceRecord, SubscriptionAdapter, benchmark_case_ids};
use mind_domain::{
    AdmissionDecision, DeliberationScheduler, FactionTrigger, MindAggregate, SchedulerBounds,
};
use mind_orchestration::{DeliberationRunner, ProviderFailure, RunContext, RunnerOutcome};
use std::path::Path;

#[expect(
    clippy::result_unit_err,
    reason = "CLI reports only a redacted failure status"
)]
pub fn run_cli<P: BenchmarkProcess>(
    args: &[String],
    mut process: P,
) -> Result<Vec<EvidenceRecord>, ()> {
    let corpus = explicit_corpus(args)?;
    benchmark_case_ids(&corpus)?
        .into_iter()
        .map(|id| run_case(&corpus, &id, &mut process))
        .collect()
}

fn explicit_corpus(args: &[String]) -> Result<std::path::PathBuf, ()> {
    if args.first().map(String::as_str) != Some("--benchmark")
        || args.len() != 3
        || args[1] != "--corpus"
    {
        return Err(());
    }
    let path = Path::new(&args[2]).canonicalize().map_err(|_| ())?;
    path.join("manifest.json")
        .is_file()
        .then_some(path)
        .ok_or(())
}

fn run_case<P: BenchmarkProcess>(
    corpus: &Path,
    id: &str,
    process: &mut P,
) -> Result<EvidenceRecord, ()> {
    let bytes =
        std::fs::read(corpus.join("fixtures").join(format!("{id}.json"))).map_err(|_| ())?;
    let fixture = BenchmarkFixture::parse(&bytes)?;
    let request = fixture.request()?;
    let faction = fixture.faction()?;
    let mut adapter = SubscriptionAdapter::for_explicit_benchmark(PayloadProcess::new(
        process,
        fixture.canonical_payload(),
    ));
    let prior = MindAggregate::empty(faction);
    let mut scheduler = DeliberationScheduler::new(SchedulerBounds::ci());
    let _ = scheduler.eligibility(faction, FactionTrigger::StrategicTick(1));
    let mut runner = DeliberationRunner::new();
    let outcome = runner.run(
        &mut adapter,
        &request,
        &prior,
        RunContext {
            current_snapshot_identity: fixture.current_snapshot_identity(),
            scheduler: &mut scheduler,
            faction,
        },
    );
    if !matches!(
        outcome,
        RunnerOutcome::Admitted {
            admission: AdmissionDecision::Accepted(_),
            ..
        }
    ) {
        return Err(());
    }
    Ok(EvidenceRecord::redacted(
        request.identity(),
        request.metadata().provider_id(),
        request.metadata().model_id(),
    ))
}

struct PayloadProcess<'a, P> {
    process: &'a mut P,
    payload: String,
}
impl<'a, P> PayloadProcess<'a, P> {
    fn new(process: &'a mut P, payload: String) -> Self {
        Self { process, payload }
    }
}
impl<P: BenchmarkProcess> BenchmarkProcess for PayloadProcess<'_, P> {
    fn invoke(&mut self, _: &str) -> Result<Vec<u8>, ProviderFailure> {
        self.process.invoke(&self.payload)
    }
}

#[cfg(test)]
#[path = "benchmark_tests.rs"]
mod benchmark_tests;
