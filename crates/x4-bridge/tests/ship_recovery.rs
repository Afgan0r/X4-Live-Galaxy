#![expect(
    clippy::expect_used,
    reason = "durable recovery fixtures fail immediately"
)]
mod carrier_b_support;
mod ship_support;
use observation_application::LifecycleResult;
use observation_domain::{BatchId, CompleteMessage, SectionKey, SectionRevisionId, TransportEpoch};
use observation_ingest::{ReceiverDisposition, decode_complete_message};
use observation_persistence::{ObservationRepository, SqliteObservationRepository};
use x4_bridge::{HeavyShipLimits, ProductionObservationSession};

fn session(path: &std::path::Path) -> ProductionObservationSession {
    let mut receiver = ProductionObservationSession::open(
        path,
        carrier_b_support::generation_limits(),
        carrier_b_support::publication_limits(),
        carrier_b_support::lifecycle_limits(),
        4,
    )
    .expect("session");
    receiver.select_ship_core("argon").expect("selection");
    receiver
        .configure_heavy(
            HeavyShipLimits::parse(include_str!("../../../config/heavy-ship-experiment.json"))
                .expect("profile"),
        )
        .expect("configure");
    receiver
}
fn publish(receiver: &mut ProductionObservationSession, revision: u64, count: usize) {
    for (ordinal, bytes) in ship_support::messages(revision, "argon")
        .into_iter()
        .take(count)
        .enumerate()
    {
        let expected = if matches!(
            decode_complete_message(&bytes, 4096).expect("message"),
            CompleteMessage::SectionCompletion(_)
        ) {
            ReceiverDisposition::Committed
        } else {
            ReceiverDisposition::Received
        };
        let result = receiver
            .submit_received(
                TransportEpoch::new(7).expect("epoch"),
                BatchId::new(format!("recovery:{revision}:{ordinal}")).expect("batch"),
                bytes,
                1,
                revision * 10 + ordinal as u64,
            )
            .expect("submit");
        assert_eq!(result, LifecycleResult::Disposition(expected));
    }
}
#[test]
fn incomplete_core_restart_never_replaces_accepted_history_or_consumes_a_durable_revision() {
    let total = ship_support::messages(2, "argon").len();
    for count in 0..total {
        let database = carrier_b_support::database(&format!("configured-incomplete-{count}"));
        let mut receiver = session(database.path());
        publish(&mut receiver, 1, usize::MAX);
        publish(&mut receiver, 2, count);
        drop(receiver);
        let mut reopened = session(database.path());
        assert_eq!(reopened.heavy_revision_floor().expect("floor"), 2);
        publish(&mut reopened, 2, usize::MAX);
        drop(reopened);
        let repository = SqliteObservationRepository::open(
            database.path(),
            carrier_b_support::publication_limits(),
        )
        .expect("independent reopen");
        let key = SectionKey::new("ship_core").expect("key");
        assert_eq!(
            repository
                .current(&key)
                .expect("current")
                .expect("core")
                .receipt()
                .revision
                .get(),
            2
        );
        assert!(
            repository
                .stored_revision(&key, SectionRevisionId::new(1).expect("revision"))
                .expect("earlier")
                .is_some()
        );
    }
}
#[test]
fn configured_retention_keeps_current_and_two_prior_receipts_and_exact_replay() {
    let database = carrier_b_support::database("configured-retention");
    let mut receiver = session(database.path());
    for revision in 1..=4 {
        publish(&mut receiver, revision, usize::MAX);
    }
    assert_eq!(
        receiver
            .maintain_heavy_history()
            .expect("retention")
            .deleted_revisions,
        1
    );
    drop(receiver);
    let repository =
        SqliteObservationRepository::open(database.path(), carrier_b_support::publication_limits())
            .expect("independent reopen");
    let key = SectionKey::new("ship_core").expect("key");
    for revision in 1..=4 {
        assert_eq!(
            repository
                .stored_revision(&key, SectionRevisionId::new(revision).expect("revision"))
                .expect("history")
                .is_some(),
            revision != 1
        );
    }
    drop(repository);
    let mut reopened = session(database.path());
    assert_eq!(reopened.heavy_revision_floor().expect("floor"), 5);
    publish(&mut reopened, 4, usize::MAX);
    assert_eq!(reopened.heavy_revision_floor().expect("replay floor"), 5);
}
