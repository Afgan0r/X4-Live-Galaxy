mod carrier_b_support;

use observation_application::{LifecycleError, LifecycleResult, ReconcileResult};
use observation_ingest::{DecisionEligibility, ReceiverDisposition};
use observation_persistence::SqliteObservationRepository;

use carrier_b_support::{
    AmbiguousRepository, FirstPublish, completion_bytes, context, database, input,
    publication_limits, session_with, start_bytes,
};

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "recovery test keeps the ordered ambiguity and reconciliation assertions together"
)]
fn rejected_boundary_preserves_ambiguous_reconciliation() {
    let database = database("boundary-during-response-loss");
    let repository = SqliteObservationRepository::open(database.path(), publication_limits())
        .expect("repository opens");
    let mut session = session_with(AmbiguousRepository::new(
        repository,
        FirstPublish::CommitThenAmbiguous,
    ));
    assert_eq!(
        session.submit(input(
            "outer:start",
            start_bytes("runtime-clock"),
            observation_application::LifecycleContext::Start(context(
                observation_domain::SectionCoverage::KnownEmpty,
            )),
            1,
        )),
        Ok(LifecycleResult::Disposition(ReceiverDisposition::Received))
    );
    assert_eq!(
        session.submit(input(
            "outer:complete",
            completion_bytes("runtime-clock"),
            observation_application::LifecycleContext::Completion(carrier_b_support::current()),
            2,
        )),
        Ok(LifecycleResult::Disposition(
            ReceiverDisposition::AmbiguousCommit
        ))
    );
    let boundary = String::from_utf8(start_bytes("runtime-clock"))
        .expect("fixture is UTF-8")
        .replacen(
            "\"producer_incarnation\":\"producer:1\"",
            "\"producer_incarnation\":\"producer:2\"",
            1,
        )
        .replacen("\"transport_epoch\":1", "\"transport_epoch\":2", 1)
        .replacen("\"section_revision\":1", "\"section_revision\":2", 1)
        .replacen(
            "\"source_epoch_status\":\"unknown\"",
            "\"source_epoch_status\":\"boundary_uncertain\"",
            1,
        )
        .replacen(
            "\"source_boundary\":\"unknown\"",
            "\"source_boundary\":\"lua_reload\"",
            1,
        )
        .into_bytes();
    assert_eq!(
        session.submit_received(
            observation_domain::TransportEpoch::new(2).expect("epoch"),
            observation_domain::BatchId::new("outer:rejected-boundary").expect("identity"),
            boundary,
            0,
            3,
        ),
        Err(x4_bridge::ProductionError::Lifecycle(
            LifecycleError::BlockedAmbiguous
        ))
    );
    assert_eq!(
        session.reconcile_ambiguous(3),
        Ok(LifecycleResult::Reconciled(ReconcileResult::Committed))
    );
    let key = observation_domain::SectionKey::new("runtime-clock").expect("key");
    assert!(matches!(
        session.decision_eligibility(&[key], 3, 10),
        DecisionEligibility::Eligible(_)
    ));
}
