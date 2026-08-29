use mind_domain::{AdmissionDecision, MindAggregate, ShadowProposal, admit};
use strategic_state::{Capability, Faction};

use super::{proposal, request};

#[test]
fn sd_004_unsupported_capability_is_rejected_without_a_pending_commit() {
    let (Ok(request), Ok(candidate)) = (
        request(),
        serde_json::to_vec(&ShadowProposal::new(
            "schema-v1",
            Capability::EconomyAndLogistics,
            1,
            "short",
            ["ZYA:military:fleet"],
            ["preserve logistics"],
            "hold the visible frontier",
            "mind-zya-shadow-unsupported",
        )),
    ) else {
        panic!("canonical semantic-rejection fixture must be constructible");
    };
    let prior = MindAggregate::empty(Faction::Zya);
    let decision = admit(&request, &prior, request.snapshot_identity(), &candidate);
    assert_eq!(
        decision,
        AdmissionDecision::Rejected(mind_domain::AdmissionRejection::Safety)
    );
    assert!(decision.pending_commit(&prior).is_none());
}

#[test]
fn sd_005_oversized_candidate_and_tradeoffs_are_rejected_atomically() {
    let (Ok(request), Ok(candidate)) = (request(), proposal()) else {
        panic!("canonical budget fixture must be constructible");
    };
    let prior = MindAggregate::empty(Faction::Zya);
    let mut oversized_candidate = candidate;
    oversized_candidate.extend(std::iter::repeat_n(b' ', 2048));
    assert_eq!(
        admit(
            &request,
            &prior,
            request.snapshot_identity(),
            &oversized_candidate
        ),
        AdmissionDecision::Rejected(mind_domain::AdmissionRejection::Oversized)
    );
    let too_many_tradeoffs = serde_json::to_vec(&ShadowProposal::new(
        "schema-v1",
        Capability::DefenseAndMilitaryStrategy,
        1,
        "short",
        ["ZYA:military:fleet"],
        ["one", "two", "three", "four", "five"],
        "hold the visible frontier",
        "mind-zya-shadow-budget",
    ));
    assert!(too_many_tradeoffs.is_ok());
    let Ok(too_many_tradeoffs) = too_many_tradeoffs else {
        return;
    };
    assert_eq!(
        admit(
            &request,
            &prior,
            request.snapshot_identity(),
            &too_many_tradeoffs
        ),
        AdmissionDecision::Rejected(mind_domain::AdmissionRejection::Budget)
    );
    assert_eq!(prior, MindAggregate::empty(Faction::Zya));
}
