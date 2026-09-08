use std::num::NonZeroUsize;

use observation_application::LifecycleLimits;
use observation_domain::{SourceScopeId, TransportEpoch};
use observation_ingest::{
    AggregateLimits, CandidateLimits, CarrierIdentity, GenerationLimits, ReceiverDisposition,
};
use observation_persistence::PublicationLimits;

use super::*;

fn start_bytes() -> Vec<u8> {
    br#"{"type":"section_start","contract_version":2,"source_scope":"scope:x4","producer_incarnation":"producer:1","transport_epoch":1,"section_key":"runtime-clock","section_revision":1,"expected_records":0,"sender_evidence":{"capture_clock":"game_time_millis","capture_start_millis":0,"capture_end_millis":0,"freshness":"fresh","quality":"unknown","availability":"available","coverage":"complete","source_epoch":null,"source_epoch_status":"unknown","source_boundary":"unknown","source_consistency":"unknown","stable_identity":false,"schema_version":1,"policy_version":2,"canonicalization_version":3,"digest_version":1}}"#.to_vec()
}

fn session(path: &std::path::Path) -> ProductionObservationSession {
    let candidate = CandidateLimits::new(4_096, 4, 4, 8, 1_000, 4).expect("candidate limits");
    let aggregate = AggregateLimits::new(2, 8_192, 8, 8, 16).expect("aggregate limits");
    ProductionObservationSession::open(
        path,
        GenerationLimits::bounded(candidate, aggregate),
        PublicationLimits::new(4, 4_096).expect("publication limits"),
        LifecycleLimits::new(4_096, 8_192, 1_000, 4).expect("lifecycle limits"),
        NonZeroUsize::new(4).map_or(4, NonZeroUsize::get),
    )
    .expect("production session")
}

#[test]
fn first_start_response_loss_retains_scope_for_candidate_invalidation() {
    let root = std::env::temp_dir().join(format!("response-loss-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("test directory");
    let mut history = OperationalHistory::open(&root).expect("history");
    let database = root.join("observations.sqlite3");
    let mut session = session(&database);
    let identity = CarrierIdentity {
        session_id: "session-1".to_owned(),
        producer_incarnation: "producer:1".to_owned(),
        epoch: TransportEpoch::new(1).expect("epoch"),
    };
    let bytes = start_bytes();

    let error = admit(&identity, &bytes, 4_096, &mut history, &mut session, |_| {
        Err(())
    })
    .expect_err("disposition response is lost");
    let (scope, disposition) = error.response_loss().expect("scope survives response loss");
    assert_eq!(scope, &SourceScopeId::new("scope:x4").expect("scope"));
    assert_eq!(disposition, ReceiverDisposition::Received);
    session.invalidate_source_scope(scope);

    let retried = admit(&identity, &bytes, 4_096, &mut history, &mut session, |_| {
        Ok(())
    });
    assert!(matches!(retried, Ok((_, ReceiverDisposition::Received))));
    let _ = std::fs::remove_dir_all(root);
}

#[test]
fn committed_response_loss_is_classified_as_ambiguous_delivery() {
    let root = std::env::temp_dir().join(format!("committed-loss-{}", std::process::id()));
    let mut history = OperationalHistory::open(&root).expect("history");
    history.bind_session("session-1", 1);
    history.bind_message("completion-1", "runtime-clock", 1);
    assert!(history.record(
        response_loss_state(ReceiverDisposition::Committed),
        "disposition-response-loss"
    ));
    let events = std::fs::read_to_string(root.join("operational-history.jsonl")).expect("events");
    assert!(events.contains("\"state\":\"ambiguous\",\"reason\":\"disposition-response-loss\""));
    assert!(!events.contains("\"state\":\"rejected\",\"reason\":\"disposition-response-loss\""));
    assert_eq!(
        response_loss_state(ReceiverDisposition::Received),
        "rejected"
    );
    let _ = std::fs::remove_dir_all(root);
}
