use mind_domain::{
    DeliberationRequest, DeliberationScheduler, FactionTrigger, SchedulerBounds, ShadowProposal,
};
use mind_orchestration::{
    EvidenceClass, ProviderFailure, ProviderMetadata, ProviderRequest, RunContext, ShadowProvider,
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
    fn evidence(&self) -> EvidenceClass {
        EvidenceClass::DeterministicFixture
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
fn canonical(identity: &str, request: DeliberationRequest) -> Result<ProviderRequest, String> {
    ProviderRequest::new(
        identity,
        1,
        request,
        ProviderMetadata::new("fixture", "fake-v1")
            .map_err(|error| format!("metadata: {error:?}"))?,
    )
    .map_err(|error| format!("canonical request: {error:?}"))
}
fn scheduled() -> DeliberationScheduler {
    let mut scheduler = DeliberationScheduler::new(SchedulerBounds::ci());
    let _ = scheduler.eligibility(Faction::Zya, FactionTrigger::StrategicTick(1));
    scheduler
}
const fn context(scheduler: &mut DeliberationScheduler) -> RunContext<'_> {
    RunContext {
        current_snapshot_identity: "snapshot-zya-1",
        scheduler,
        faction: Faction::Zya,
    }
}
#[path = "provider_contract/paths.rs"]
mod paths;
#[path = "provider_contract/stale.rs"]
mod stale;
