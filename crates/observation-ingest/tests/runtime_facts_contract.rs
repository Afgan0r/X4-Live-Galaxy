use observation_ingest::{
    AcceptedProjection, AdmissionError, ReceiptClock, admit_batch, admit_batch_with_receipt_clock,
};

const V2_FACTS: &str = r#"{
  "type":"observation",
  "scope":"runtime:sectors",
  "entity_id":"sector:argon_prime",
  "version":1,
  "quality":"fresh",
  "runtime_facts":{"r":"x4_runtime","g":42,"q":"fresh","a":"available","s":[{"i":"sector:argon_prime"}],"x":[{"i":"asset:ship:1","p":"sector:argon_prime"}],"c":[{"i":"capacity:ship:storage","p":"asset:ship:1","v":42}],"o":[{"i":"ownership:ship:1","p":"asset:ship:1","n":"faction:argon"}]},
  "generation":1,
  "sequence":1
}"#;

const MARKER: &str = r#"{"type":"complete_marker","scope":"runtime:sectors","version":1,"generation":1,"sequence":2}"#;

#[path = "runtime_facts_contract/station_contract.rs"]
mod station_contract;

struct FixedClock(u64);

impl ReceiptClock for FixedClock {
    fn receipt_unix_millis(&self) -> Result<u64, AdmissionError> {
        Ok(self.0)
    }
}

struct FailingClock;

impl ReceiptClock for FailingClock {
    fn receipt_unix_millis(&self) -> Result<u64, AdmissionError> {
        Err(AdmissionError::ReceiptClockUnavailable)
    }
}

#[test]
fn canonical_v2_runtime_facts_remain_inspectable_after_admission() {
    let accepted = admit_batch_with_receipt_clock(
        AcceptedProjection::empty(),
        &[V2_FACTS, MARKER],
        &FixedClock(1_725_000_000_000),
    )
    .into_projection();

    let facts = accepted
        .runtime_facts("sector:argon_prime")
        .expect("the accepted sector retains its v2 facts");
    assert_eq!(facts.sectors.len(), 1);
    assert_eq!(facts.assets.len(), 1);
    assert_eq!(facts.capacity.len(), 1);
    assert_eq!(facts.ownership.len(), 1);
    assert_eq!(facts.x4_game_time, Some(42));
    assert_eq!(facts.receipt_unix_millis, 1_725_000_000_000);
}

#[test]
fn incomplete_or_oversized_fact_classes_preserve_the_prior_projection() {
    let prior = admit_batch(AcceptedProjection::empty(), &[V2_FACTS, MARKER]).into_projection();
    let incomplete = r#"{"type":"observation","scope":"runtime:sectors","entity_id":"sector:argon_prime","version":2,"quality":"fresh","runtime_facts":{"r":"x4_runtime","g":2,"q":"fresh","a":"available","s":[{"i":"sector:argon_prime"}],"x":[{"i":"asset:ship:1","p":"sector:argon_prime"}],"c":[{"i":"capacity:ship:storage","p":"asset:ship:1","v":42}],"o":[]},"generation":1,"sequence":3}"#;
    let rejected = admit_batch(prior.clone(), &[incomplete]);

    assert_eq!(rejected.snapshot(), prior.snapshot());
    assert_eq!(
        rejected.projection().runtime_facts("sector:argon_prime"),
        prior.runtime_facts("sector:argon_prime")
    );
    assert!(rejected.rejection_reason().is_some());
}

#[test]
fn client_unix_timestamp_rejects_without_replacing_prior_facts() {
    let prior = admit_batch_with_receipt_clock(
        AcceptedProjection::empty(),
        &[V2_FACTS, MARKER],
        &FixedClock(99),
    )
    .into_projection();
    let untrusted_timestamp = V2_FACTS.replace(
        "\"version\":1",
        "\"observed_at_unix_millis\":1,\"version\":1",
    );
    let rejected =
        admit_batch_with_receipt_clock(prior.clone(), &[&untrusted_timestamp], &FixedClock(100));

    assert_eq!(rejected.snapshot(), prior.snapshot());
    assert_eq!(
        rejected.projection().runtime_facts("sector:argon_prime"),
        prior.runtime_facts("sector:argon_prime")
    );
}

#[test]
fn receipt_clock_failure_rejects_after_strict_admission() {
    let rejected = admit_batch_with_receipt_clock(
        AcceptedProjection::empty(),
        &[V2_FACTS, MARKER],
        &FailingClock,
    );

    assert!(rejected.rejection_reason().is_some());
    assert!(rejected.snapshot().entity_ids().is_empty());
}

#[test]
fn duplicate_fact_identity_rejects_without_replacing_the_prior_facts() {
    let prior = admit_batch(AcceptedProjection::empty(), &[V2_FACTS, MARKER]).into_projection();
    let duplicate = V2_FACTS.replace(
        r#""s":[{"i":"sector:argon_prime"}]"#,
        r#""s":[{"i":"sector:argon_prime"},{"i":"sector:argon_prime"}]"#,
    );
    let rejected = admit_batch(prior.clone(), &[&duplicate]);

    assert_eq!(rejected.snapshot(), prior.snapshot());
    assert!(rejected.rejection_reason().is_some());
}
