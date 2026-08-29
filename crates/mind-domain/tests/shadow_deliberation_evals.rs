#[path = "shadow_deliberation_evals/exact_cache.rs"]
mod exact_cache;

use mind_domain::{
    AdmissionDecision, CommandId, DeliberationRequest, DeliberationScheduler, FactionTrigger,
    MindAggregate, RequestEligibility, SchedulerBounds, ShadowProposal, admit,
};
use observation_ingest::{AcceptedProjection, admit_batch};
use strategic_state::{Capability, Faction, PacketLimits, derive_packets};

const FRAMES: [&str; 2] = [
    r#"{"type":"observation","scope":"ZYA","entity_id":"ZYA:military:fleet","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"own"}"#,
    r#"{"type":"observation","scope":"XEN","entity_id":"XEN:threat:XEN","observed_at_unix_millis":1,"version":1,"quality":"fresh","content":"shared"}"#,
];

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

fn proposal() -> Result<Vec<u8>, String> {
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
    .map_err(|error| format!("proposal fixture: {error}"))
}

#[test]
fn sd_002_valid_candidate_creates_pending_commit() {
    let request = request();
    assert!(request.is_ok());
    let Ok(request) = request else { return };
    let proposal = proposal();
    assert!(proposal.is_ok());
    let Ok(proposal) = proposal else { return };
    let prior = MindAggregate::empty(Faction::Zya);
    let decision = admit(&request, &prior, &proposal);
    assert!(matches!(decision, AdmissionDecision::Accepted(_)));
    let AdmissionDecision::Accepted(accepted) = decision else {
        return;
    };
    let pending = accepted.pending_commit(&prior);
    assert!(pending.is_ok());
    assert_eq!(accepted.command_id(), CommandId::new("mind-zya-shadow-1"));
}

#[test]
fn sd_003_malformed_missing_and_unknown_candidates_are_rejected() {
    for candidate in [b"{".as_slice(), b"{}", b"{\"unknown\":true}"] {
        let request = request();
        assert!(request.is_ok());
        let Ok(request) = request else { return };
        assert!(matches!(
            admit(&request, &MindAggregate::empty(Faction::Zya), candidate),
            AdmissionDecision::Rejected(_)
        ));
    }
}

#[test]
fn sd_001_hidden_and_sd_013_forbidden_candidates_have_no_pending_projection() {
    let hidden = br#"{"schema_version":"schema-v1","capability":"DefenseAndMilitaryStrategy","priority":1,"horizon":"short","supporting_fact_ids":["ARG:secret"],"trade_offs":["preserve logistics"],"explanation":"hold frontier","command_id":"mind-zya-shadow-1"}"#;
    let forbidden = br#"{"schema_version":"schema-v1","capability":"EconomyAndLogistics","priority":1,"horizon":"short","supporting_fact_ids":["ZYA:military:fleet"],"trade_offs":["preserve logistics"],"explanation":"hold frontier","command_id":"mind-zya-shadow-1"}"#;
    for candidate in [hidden.as_slice(), forbidden.as_slice()] {
        let request = request();
        assert!(request.is_ok());
        let Ok(request) = request else { return };
        let prior = MindAggregate::empty(Faction::Zya);
        let decision = admit(&request, &prior, candidate);
        assert!(matches!(decision, AdmissionDecision::Rejected(_)));
        assert!(decision.pending_commit(&prior).is_none());
    }
}

#[test]
fn stale_faction_is_rejected_before_any_pending_commit() {
    let request = request();
    assert!(request.is_ok());
    let Ok(request) = request else { return };
    let proposal = proposal();
    assert!(proposal.is_ok());
    let Ok(proposal) = proposal else { return };
    let decision = admit(&request, &MindAggregate::empty(Faction::Arg), &proposal);
    assert_eq!(
        decision,
        AdmissionDecision::Rejected(mind_domain::AdmissionRejection::CurrentState)
    );
    assert!(
        decision
            .pending_commit(&MindAggregate::empty(Faction::Arg))
            .is_none()
    );
}

#[test]
fn sd_007_duplicate_and_interrupted_triggers_keep_one_faction_owner() {
    let mut scheduler = DeliberationScheduler::new(SchedulerBounds::ci());
    let first = scheduler.eligibility(Faction::Zya, FactionTrigger::StrategicTick(1));
    assert!(matches!(first, RequestEligibility::Eligible(_)));
    for trigger in [
        FactionTrigger::StrategicTick(1),
        FactionTrigger::RelevantEvent("XEN:threat:XEN".into()),
        FactionTrigger::Interrupted,
    ] {
        assert_eq!(
            scheduler.eligibility(Faction::Zya, trigger),
            RequestEligibility::Coalesced
        );
    }
    assert_eq!(scheduler.outstanding_count(Faction::Zya), 1);
}

#[test]
fn sd_011_timeout_pauses_until_a_newer_reconciled_observation() {
    let mut scheduler = DeliberationScheduler::new(SchedulerBounds::ci());
    assert!(matches!(
        scheduler.eligibility(Faction::Arg, FactionTrigger::StrategicTick(4)),
        RequestEligibility::Eligible(_)
    ));
    assert_eq!(
        scheduler.timeout(Faction::Arg, 4),
        RequestEligibility::PausedAwaitingReconciliation
    );
    assert_eq!(
        scheduler.reconcile(Faction::Arg, 4),
        RequestEligibility::PausedAwaitingReconciliation
    );
    assert!(matches!(
        scheduler.reconcile(Faction::Arg, 5),
        RequestEligibility::Reconciled
    ));
}
