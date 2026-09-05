mod support;

use observation_application::{LifecycleContext, LifecycleResult, ObservationLifecycle};
use observation_domain::SectionCoverage;
use observation_ingest::{DecisionRevisionIndex, ReceiverDisposition};
use support::flow::{input, limits};
use support::{candidate_context, repository, stager, start_bytes};

#[test]
fn exact_replay_rejects_changed_semantic_context_without_dispatch() {
    let (_database, repository) = repository("semantic-replay");
    let mut lifecycle = ObservationLifecycle::new(
        stager(),
        DecisionRevisionIndex::new(4).expect("blocker limit is non-zero"),
        repository,
        limits(),
    );
    let bytes = start_bytes("stations", 0);
    assert_eq!(
        lifecycle.submit(input(
            "outer:stations:start",
            bytes.clone(),
            LifecycleContext::Start(candidate_context(SectionCoverage::KnownEmpty)),
            1,
        )),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Received))
    );
    assert_eq!(
        lifecycle.submit(input(
            "outer:stations:start",
            bytes.clone(),
            LifecycleContext::Start(candidate_context(SectionCoverage::Complete)),
            2,
        )),
        Ok(LifecycleResult::Disposition(
            ReceiverDisposition::PermanentlyRejected
        ))
    );
    assert_eq!(
        lifecycle.submit(input(
            "outer:stations:start",
            bytes,
            LifecycleContext::Start(candidate_context(SectionCoverage::KnownEmpty)),
            3,
        )),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Received))
    );
}
