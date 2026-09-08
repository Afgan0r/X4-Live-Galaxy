use std::num::NonZeroUsize;

use observation_application::LifecycleLimits;
use observation_domain::{SourceScopeId, TransportEpoch};
use observation_ingest::{
    AggregateLimits, CandidateLimits, CarrierIdentity, DecisionEligibility, GenerationLimits,
    ReceiverDisposition,
};
use observation_persistence::PublicationLimits;

use super::*;

fn start_bytes() -> Vec<u8> {
    br#"{"type":"section_start","contract_version":2,"source_scope":"scope:x4","producer_incarnation":"producer:1","transport_epoch":1,"section_key":"runtime-clock","section_revision":1,"expected_records":0,"sender_evidence":{"capture_clock":"game_time_millis","capture_start_millis":0,"capture_end_millis":0,"freshness":"fresh","quality":"unknown","availability":"available","coverage":"complete","source_epoch":null,"source_epoch_status":"unknown","source_boundary":"unknown","source_consistency":"unknown","stable_identity":false,"schema_version":1,"policy_version":2,"canonicalization_version":3,"digest_version":1}}"#.to_vec()
}

fn completion_bytes() -> Vec<u8> {
    br#"{"type":"section_completion","contract_version":2,"source_scope":"scope:x4","producer_incarnation":"producer:1","transport_epoch":1,"section_key":"runtime-clock","section_revision":1,"batch_count":0,"record_count":0,"raw_bytes":0,"ordered_batch_manifest_digest":"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855","canonical_content_digest":"e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855","schema_version":1,"policy_version":2,"canonicalization_version":3,"digest_version":1,"coverage":"known_empty","sender_evidence":{"capture_clock":"game_time_millis","capture_start_millis":0,"capture_end_millis":0,"freshness":"fresh","quality":"fresh","availability":"available","coverage":"known_empty","source_epoch":null,"source_epoch_status":"unknown","source_boundary":"unknown","source_consistency":"unknown","stable_identity":true,"schema_version":1,"policy_version":2,"canonicalization_version":3,"digest_version":1}}"#.to_vec()
}

fn qualifying_start_bytes() -> Vec<u8> {
    String::from_utf8(start_bytes())
        .expect("fixture is UTF-8")
        .replacen("\"quality\":\"unknown\"", "\"quality\":\"fresh\"", 1)
        .replacen(
            "\"coverage\":\"complete\"",
            "\"coverage\":\"known_empty\"",
            1,
        )
        .replacen("\"stable_identity\":false", "\"stable_identity\":true", 1)
        .into_bytes()
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

#[test]
#[expect(
    clippy::too_many_lines,
    reason = "forgery test keeps durable authority and volatile admission assertions together"
)]
fn forged_carrier_identity_preserves_authority_and_volatile_admission_state() {
    let root = std::env::temp_dir().join(format!("forged-admission-{}", std::process::id()));
    std::fs::create_dir_all(&root).expect("test directory");
    let mut history = OperationalHistory::open(&root).expect("history");
    let mut session = session(&root.join("observations.sqlite3"));
    let identity = CarrierIdentity {
        session_id: "session-1".to_owned(),
        producer_incarnation: "producer:1".to_owned(),
        epoch: TransportEpoch::new(1).expect("epoch"),
    };
    assert!(
        admit(
            &identity,
            &qualifying_start_bytes(),
            4_096,
            &mut history,
            &mut session,
            |_| Ok(())
        )
        .is_ok()
    );
    assert!(
        admit(
            &identity,
            &completion_bytes(),
            4_096,
            &mut history,
            &mut session,
            |_| Ok(())
        )
        .is_ok()
    );
    let key = observation_domain::SectionKey::new("runtime-clock").expect("section key");
    let before = session.decision_eligibility(std::slice::from_ref(&key), u64::MAX, u64::MAX);
    assert!(matches!(before, DecisionEligibility::Eligible(_)));

    let next = String::from_utf8(qualifying_start_bytes())
        .expect("fixture is UTF-8")
        .replacen("\"section_revision\":1", "\"section_revision\":2", 1);
    let forged = [
        next.replacen(
            "\"producer_incarnation\":\"producer:1\"",
            "\"producer_incarnation\":\"producer:forged\"",
            1,
        ),
        next.replacen("\"transport_epoch\":1", "\"transport_epoch\":2", 1),
        r#"{"type":"immutable_batch","contract_version":2,"source_scope":"scope:x4","producer_incarnation":"producer:forged","transport_epoch":1,"section_key":"runtime-clock","section_revision":2,"batch_id":"inner:forged-producer","section_ordinal":1,"records":[{"record_id":"record:1","entity_id":"entity:1","observation_version":1,"content":"value"}],"optional_detail":null}"#.to_owned(),
        r#"{"type":"immutable_batch","contract_version":2,"source_scope":"scope:x4","producer_incarnation":"producer:1","transport_epoch":2,"section_key":"runtime-clock","section_revision":2,"batch_id":"inner:forged-epoch","section_ordinal":1,"records":[{"record_id":"record:1","entity_id":"entity:1","observation_version":1,"content":"value"}],"optional_detail":null}"#.to_owned(),
    ];
    for bytes in forged {
        let mut response_called = false;
        let result = admit(
            &identity,
            bytes.as_bytes(),
            4_096,
            &mut history,
            &mut session,
            |_| {
                response_called = true;
                Ok(())
            },
        );
        assert!(matches!(result, Err(AdmitError::Identity)));
        assert!(!response_called);
        assert_eq!(
            session.decision_eligibility(std::slice::from_ref(&key), u64::MAX, u64::MAX),
            before
        );
    }

    assert!(matches!(
        admit(
            &identity,
            next.as_bytes(),
            4_096,
            &mut history,
            &mut session,
            |_| Ok(())
        ),
        Ok((_, ReceiverDisposition::Received))
    ));
    let _ = std::fs::remove_dir_all(root);
}
