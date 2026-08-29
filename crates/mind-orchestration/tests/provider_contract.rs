use mind_domain::{DeliberationRequest, MindAggregate, ShadowProposal};
use mind_orchestration::{
    DeliberationRunner, EvidenceClass, ProviderFailure, ProviderMetadata, ProviderRequest,
    RunnerOutcome, ShadowProvider,
};
use observation_ingest::{AcceptedProjection, admit_batch};
use strategic_state::{Capability, Faction, PacketLimits, derive_packets};

const FRAMES: [&str; 2] = [
    r#"{"type":"observation","scope":"ZYA","entity_id":"ZYA:military:fleet","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#,
    r#"{"type":"observation","scope":"XEN","entity_id":"XEN:threat:XEN","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"shared"}"#,
];

struct FakeProvider {
    outcome: Result<Vec<u8>, ProviderFailure>,
}

impl ShadowProvider for FakeProvider {
    fn propose(&mut self, _: &ProviderRequest) -> Result<Vec<u8>, ProviderFailure> {
        self.outcome.clone()
    }
}

fn request() -> Result<DeliberationRequest, String> {
    let snapshot = admit_batch(AcceptedProjection::empty(), &FRAMES)
        .into_projection()
        .snapshot;
    let packets = derive_packets(&snapshot, PacketLimits::tracer())
        .map_err(|error| format!("packet fixture: {error:?}"))?;
    DeliberationRequest::from_packet(
        packets.packet(Faction::Zya).clone(),
        "snapshot-zya-1",
        ["ZYA:military:fleet", "XEN:threat:XEN"],
        [Capability::DefenseAndMilitaryStrategy],
        "policy-v1",
        "prompt-v1",
        2048,
        4,
    )
    .map_err(|error| format!("frozen request: {error:?}"))
}

fn candidate() -> Result<Vec<u8>, String> {
    serde_json::to_vec(&ShadowProposal::new(
        "schema-v1",
        Capability::DefenseAndMilitaryStrategy,
        1,
        "short",
        ["ZYA:military:fleet"],
        ["preserve logistics"],
        "hold the visible frontier",
        "mind-zya-shadow-1",
    ))
    .map_err(|error| format!("candidate fixture: {error}"))
}

#[test]
fn deterministic_fake_replays_candidate_through_shared_admission() {
    let request = request();
    let candidate = candidate();
    assert!(request.is_ok() && candidate.is_ok());
    let (Ok(request), Ok(candidate)) = (request, candidate) else {
        return;
    };
    let canonical = ProviderRequest::new(
        "request-zya-1",
        request,
        ProviderMetadata::new("fixture", "fake-v1").unwrap(),
    )
    .unwrap();
    let mut first = FakeProvider { outcome: Ok(candidate.clone()) };
    let mut replay = FakeProvider { outcome: Ok(candidate) };
    let mut runner = DeliberationRunner::new();
    let prior = MindAggregate::empty(Faction::Zya);
    let first = runner.run(&mut first, &canonical, &prior);
    let replay = runner.run(&mut replay, &canonical, &prior);
    assert_eq!(first, replay);
    assert!(matches!(first, RunnerOutcome::Admitted { evidence: EvidenceClass::DeterministicFixture, .. }));
}

#[test]
fn provider_timeout_pauses_until_newer_reconciled_observation() {
    let request = request();
    assert!(request.is_ok());
    let Ok(request) = request else { return };
    let canonical = ProviderRequest::new(
        "request-zya-timeout",
        request,
        ProviderMetadata::new("fixture", "fake-v1").unwrap(),
    )
    .unwrap();
    let mut provider = FakeProvider { outcome: Err(ProviderFailure::Timeout) };
    let mut runner = DeliberationRunner::new();
    let prior = MindAggregate::empty(Faction::Zya);
    let outcome = runner.run(&mut provider, &canonical, &prior);
    let RunnerOutcome::Degraded(record) = outcome else { return };
    assert_eq!(record.evidence(), EvidenceClass::DeterministicFixture);
    assert!(record.reconcile(1).is_err());
    assert!(record.reconcile(2).is_ok());
}
