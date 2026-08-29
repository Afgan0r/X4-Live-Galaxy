use mind_domain::{
    AdmissionDecision, ExactCacheKey, MindAggregate, RequestBounds, revalidate_cached,
};
use strategic_state::Faction;

use super::{proposal, request};

fn exact_key(values: [&str; 10]) -> Result<ExactCacheKey, String> {
    ExactCacheKey::for_test(
        values[0], values[1], values[2], values[3], values[4], values[5], values[6], values[7],
        values[8], values[9],
    )
    .map_err(|error| format!("valid exact key: {error:?}"))
}

#[test]
fn sd_006_exact_key_changes_for_each_authority_component() {
    let baseline = exact_key([
        "ZYA",
        "snapshot-zya-1",
        "policy-v1",
        "prompt-v1",
        "schema-v1",
        "provider-a",
        "model-a",
        "temperature=0",
        "defense,economy",
        "compact-v1",
    ]);
    assert!(baseline.is_ok());
    let Ok(baseline) = baseline else { return };
    for index in 0..10 {
        let mut changed = [
            "ZYA",
            "snapshot-zya-1",
            "policy-v1",
            "prompt-v1",
            "schema-v1",
            "provider-a",
            "model-a",
            "temperature=0",
            "defense,economy",
            "compact-v1",
        ];
        changed[index] = [
            "ARG",
            "snapshot-2",
            "policy-2",
            "prompt-2",
            "schema-2",
            "provider-b",
            "model-b",
            "temperature=1",
            "defense",
            "compact-2",
        ][index];
        let changed = exact_key(changed);
        assert!(changed.is_ok());
        let Ok(changed) = changed else { return };
        assert_ne!(baseline, changed);
    }
}

#[test]
fn sd_006_exact_key_is_stable_for_permuted_collections() {
    let request = request();
    assert!(request.is_ok());
    let Ok(request) = request else { return };
    let bounds = RequestBounds::test_profile();
    assert!(bounds.is_ok());
    let Ok(bounds) = bounds else { return };
    let ordered = ExactCacheKey::from_request(
        &request,
        &bounds,
        "schema-v1",
        "provider-a",
        "model-a",
        ["temperature=0", "top_p=1"],
        ["defense", "economy"],
        "compact-v1",
    );
    assert!(ordered.is_ok());
    let Ok(ordered) = ordered else { return };
    let permuted = ExactCacheKey::from_request(
        &request,
        &bounds,
        "schema-v1",
        "provider-a",
        "model-a",
        ["top_p=1", "temperature=0"],
        ["economy", "defense"],
        "compact-v1",
    );
    assert!(permuted.is_ok());
    let Ok(permuted) = permuted else { return };
    assert_eq!(ordered, permuted);
}

#[test]
fn sd_005_all_request_bounds_are_required_and_nonzero() {
    for index in 0..9 {
        let mut values = [1; 9];
        values[index] = 0;
        assert!(
            RequestBounds::new(
                values[0], values[1], values[2], values[3], values[4], values[5], values[6],
                values[7], values[8]
            )
            .is_err()
        );
    }
}

#[test]
fn sd_006_cached_bytes_revalidate_through_current_state_admission() {
    let request = request();
    let candidate = proposal();
    assert!(request.is_ok());
    assert!(candidate.is_ok());
    let (Ok(request), Ok(candidate)) = (request, candidate) else {
        return;
    };
    let result = revalidate_cached(
        &request,
        &MindAggregate::empty(Faction::Arg),
        request.snapshot_identity(),
        &candidate,
    );
    assert_eq!(
        result.decision,
        AdmissionDecision::Rejected(mind_domain::AdmissionRejection::CurrentState)
    );
    assert!(result.cache_hit);
    assert_eq!(result.validator_outcome, "current_state");
}

#[test]
fn same_faction_changed_snapshot_is_rejected_without_a_pending_commit() {
    let (Ok(request), Ok(candidate)) = (request(), proposal()) else {
        return;
    };
    let prior = MindAggregate::empty(Faction::Zya);
    let result = revalidate_cached(&request, &prior, "snapshot-zya-2", &candidate);
    assert_eq!(
        result.decision,
        AdmissionDecision::Rejected(mind_domain::AdmissionRejection::CurrentState)
    );
    assert!(result.decision.pending_commit(&prior).is_none());
}
