mod support;

use observation_application::{LifecycleContext, LifecycleResult, ObservationLifecycle};
use observation_ingest::{DecisionRevisionIndex, ReceiverDisposition};
use support::flow::{input, limits};
use support::versioned;
use support::{repository, revision, stager};

#[test]
fn committed_entity_versions_reject_regression_and_conflict_then_accept_advance() {
    let (_database, repository) = repository("monotonic-application");
    let mut lifecycle = ObservationLifecycle::new(
        stager(),
        DecisionRevisionIndex::new(2).expect("blocker limit is non-zero"),
        repository,
        limits(),
    );
    submit(
        &mut lifecycle,
        2,
        2,
        "content:v2",
        None,
        ReceiverDisposition::Committed,
    );
    reject_batch(&mut lifecycle, 1, 1, "content:v1");
    reject_batch(&mut lifecycle, 3, 2, "content:changed");
    submit(
        &mut lifecycle,
        3,
        3,
        "content:v3",
        Some(revision(2)),
        ReceiverDisposition::Committed,
    );
}

fn reject_batch(
    lifecycle: &mut ObservationLifecycle<observation_persistence::SqliteObservationRepository>,
    section_revision: u64,
    entity_version: u64,
    content: &str,
) {
    let (start, lifecycle_context) = versioned::start(section_revision, Some(revision(2)));
    disposition(
        lifecycle,
        &format!("start:{section_revision}"),
        start,
        lifecycle_context,
        ReceiverDisposition::Received,
    );
    disposition(
        lifecycle,
        &format!("batch:{section_revision}"),
        versioned::batch(section_revision, entity_version, content),
        LifecycleContext::Batch,
        ReceiverDisposition::PermanentlyRejected,
    );
}

fn submit(
    lifecycle: &mut ObservationLifecycle<observation_persistence::SqliteObservationRepository>,
    section_revision: u64,
    entity_version: u64,
    content: &str,
    expected: Option<observation_domain::SectionRevisionId>,
    terminal: ReceiverDisposition,
) {
    let (start, lifecycle_context) = versioned::start(section_revision, expected);
    disposition(
        lifecycle,
        &format!("start:{section_revision}"),
        start,
        lifecycle_context,
        ReceiverDisposition::Received,
    );
    disposition(
        lifecycle,
        &format!("batch:{section_revision}"),
        versioned::batch(section_revision, entity_version, content),
        LifecycleContext::Batch,
        ReceiverDisposition::Received,
    );
    disposition(
        lifecycle,
        &format!("complete:{section_revision}"),
        versioned::completion(section_revision, entity_version, content),
        versioned::current(expected),
        terminal,
    );
}

fn disposition(
    lifecycle: &mut ObservationLifecycle<observation_persistence::SqliteObservationRepository>,
    identity: &str,
    bytes: Vec<u8>,
    context: LifecycleContext,
    expected: ReceiverDisposition,
) {
    assert_eq!(
        lifecycle.submit(input(identity, bytes, context, 1)),
        Ok(LifecycleResult::Disposition(expected))
    );
}
