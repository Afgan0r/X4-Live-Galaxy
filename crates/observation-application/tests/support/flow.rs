use observation_application::{
    LifecycleContext, LifecycleInput, LifecycleLimits, LifecycleResult, ObservationLifecycle,
};
use observation_domain::SectionCoverage;
use observation_ingest::ReceiverDisposition;
use observation_persistence::ObservationRepository;

use super::{
    batch_bytes, batch_id, candidate_context, completion_bytes, current, epoch, start_bytes,
};

pub fn limits() -> LifecycleLimits {
    LifecycleLimits::new(4_096, 16_384, 1_000, 4).expect("limits are non-zero")
}

pub fn input(
    identity: &str,
    bytes: Vec<u8>,
    context: LifecycleContext,
    now: u64,
) -> LifecycleInput {
    LifecycleInput::new(epoch(), batch_id(identity), bytes, 1, now, context)
}

pub fn submit_section<R: ObservationRepository>(
    lifecycle: &mut ObservationLifecycle<R>,
    section: &str,
    records: &[(&str, &str)],
) {
    submit_start_and_batch(lifecycle, section, records);
    assert_eq!(
        lifecycle.submit(input(
            &format!("outer:{section}:complete"),
            completion_bytes(section, records, "complete"),
            LifecycleContext::Completion(current()),
            3,
        )),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Committed))
    );
}

pub fn submit_start_and_batch<R: ObservationRepository>(
    lifecycle: &mut ObservationLifecycle<R>,
    section: &str,
    records: &[(&str, &str)],
) {
    assert_eq!(
        lifecycle.submit(input(
            &format!("outer:{section}:start"),
            start_bytes(section, records.len()),
            LifecycleContext::Start(candidate_context(SectionCoverage::Complete)),
            1,
        )),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Received))
    );
    assert_eq!(
        lifecycle.submit(input(
            &format!("outer:{section}:batch"),
            batch_bytes(section, records),
            LifecycleContext::Batch,
            2,
        )),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Received))
    );
}

pub fn submit_empty<R: ObservationRepository>(
    lifecycle: &mut ObservationLifecycle<R>,
    section: &str,
) {
    assert_eq!(
        lifecycle.submit(input(
            &format!("outer:{section}:start"),
            start_bytes(section, 0),
            LifecycleContext::Start(candidate_context(SectionCoverage::KnownEmpty)),
            4,
        )),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Received))
    );
    assert_eq!(
        lifecycle.submit(input(
            &format!("outer:{section}:complete"),
            completion_bytes(section, &[], "known_empty"),
            LifecycleContext::Completion(current()),
            5,
        )),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Committed))
    );
}
