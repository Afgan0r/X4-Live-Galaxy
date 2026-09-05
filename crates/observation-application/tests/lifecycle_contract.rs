#![expect(
    clippy::expect_used,
    clippy::panic,
    reason = "contract-test setup and mismatches must fail immediately"
)]

mod support;

use observation_application::{
    LifecycleContext, LifecycleInput, LifecycleLimits, LifecycleResult, ObservationLifecycle,
};
use observation_domain::SectionCoverage;
use observation_ingest::{DecisionRevisionIndex, ReceiverDisposition};
use observation_persistence::ObservationRepository;
use support::{
    batch_bytes, batch_id, candidate_context, completion_bytes, current, epoch, repository, stager,
    start_bytes,
};

fn limits() -> LifecycleLimits {
    LifecycleLimits::new(4_096, 16_384, 1_000, 4).expect("limits are non-zero")
}

fn input(identity: &str, bytes: Vec<u8>, context: LifecycleContext, now: u64) -> LifecycleInput {
    LifecycleInput::new(epoch(), batch_id(identity), bytes, 1, now, context)
}

#[test]
fn decode_first_ship_tracer() {
    let (_database, repository) = repository("ship-tracer");
    let mut lifecycle = ObservationLifecycle::new(
        stager(),
        DecisionRevisionIndex::new(4).expect("blocker limit is non-zero"),
        repository,
        limits(),
    );
    let invalid = input(
        "outer:invalid",
        br#"{"type":"section_start"}"#.to_vec(),
        LifecycleContext::Start(candidate_context(SectionCoverage::Complete)),
        1,
    );
    assert!(lifecycle.submit(invalid).is_err());
    assert_eq!(lifecycle.candidate_count(), 0);
    assert!(!lifecycle.slot_occupied());
}

#[test]
fn ambiguous_attempt_reconciles_exact_request_without_reassembly() {
    let (_database, repository) = repository("ambiguous");
    let mut lifecycle = ObservationLifecycle::new(
        stager(),
        DecisionRevisionIndex::new(4).expect("blocker limit is non-zero"),
        repository,
        limits(),
    );
    submit_section(&mut lifecycle, "ships", &[("record:1", "ship:1")]);
    assert_eq!(lifecycle.retained_attempt_count(), 0);
    assert_eq!(lifecycle.current_revision_count(), 1);
}

#[test]
fn ship_and_known_empty_station_traverse_same_lifecycle() {
    let (_database, repository) = repository("ship-station");
    let mut lifecycle = ObservationLifecycle::new(
        stager(),
        DecisionRevisionIndex::new(4).expect("blocker limit is non-zero"),
        repository,
        limits(),
    );
    submit_section(&mut lifecycle, "ships", &[("record:1", "ship:1")]);
    submit_empty(&mut lifecycle, "stations");
    assert_eq!(lifecycle.current_revision_count(), 2);
}

#[test]
fn adjacent_identities_survive_permuted_input_with_canonical_replay() {
    let (_database, repository) = repository("permuted");
    let mut lifecycle = ObservationLifecycle::new(
        stager(),
        DecisionRevisionIndex::new(4).expect("blocker limit is non-zero"),
        repository,
        limits(),
    );
    submit_section(
        &mut lifecycle,
        "ships",
        &[("record:2", "ship:2"), ("record:1", "ship:1")],
    );
    assert_eq!(lifecycle.current_revision_count(), 1);
}

fn submit_section<R: ObservationRepository>(
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
    assert_eq!(
        lifecycle.submit(input(
            &format!("outer:{section}:complete"),
            completion_bytes(section, records.len(), "complete"),
            LifecycleContext::Completion(current()),
            3,
        )),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Committed))
    );
}

fn submit_empty<R: ObservationRepository>(lifecycle: &mut ObservationLifecycle<R>, section: &str) {
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
            completion_bytes(section, 0, "known_empty"),
            LifecycleContext::Completion(current()),
            5,
        )),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Committed))
    );
}
