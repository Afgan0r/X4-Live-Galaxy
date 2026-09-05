use observation_application::LifecycleInput;

use super::*;
use crate::support::{batch_bytes, batch_id, epoch};

#[test]
fn batch_replay_binds_exact_and_one_over_work_limits() {
    let (_database, repository) = repository("batch-work-replay");
    let mut lifecycle = ObservationLifecycle::new(
        stager(),
        DecisionRevisionIndex::new(4).expect("blocker limit is non-zero"),
        repository,
        limits(),
    );
    assert_eq!(
        lifecycle.submit(input(
            "outer:ships:start",
            start_bytes("ships", 1),
            LifecycleContext::Start(candidate_context(SectionCoverage::Complete)),
            1,
        )),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Received))
    );
    let bytes = batch_bytes("ships", &[("record:1", "ship:1")]);
    let batch_input = |work, now| {
        LifecycleInput::new(
            epoch(),
            batch_id("outer:ships:batch"),
            bytes.clone(),
            work,
            now,
            LifecycleContext::Batch,
        )
    };
    assert_eq!(
        lifecycle.submit(batch_input(32, 2)),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Received))
    );
    assert_eq!(
        lifecycle.submit(batch_input(32, 3)),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Received))
    );
    assert_eq!(
        lifecycle.submit(batch_input(33, 4)),
        Ok(LifecycleResult::Disposition(
            ReceiverDisposition::PermanentlyRejected
        ))
    );
    assert_eq!(
        lifecycle.submit(batch_input(32, 5)),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Received))
    );
}
