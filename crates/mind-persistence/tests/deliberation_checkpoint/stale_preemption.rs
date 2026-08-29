use super::*;

#[test]
fn stale_preemption_rejects_current_state_before_a_pending_or_checkpoint() {
    let (Ok(request), Ok(bytes), Ok(prior)) =
        (request(), candidate_bytes(), prior_with_active_initiative())
    else {
        return;
    };
    let Some(active) = prior.active_initiative(Capability::DefenseAndMilitaryStrategy) else {
        return;
    };
    let preemption = PreemptionRequest::new(
        CommandId::new("mind-zya-shadow-1"),
        "visible xen threat",
        active.clone(),
        PreemptionDisposition::Cancelled,
        initiative("initiative-b"),
        mind_domain::ExecutiveDecision::Approve,
        "replace the stale defense initiative",
    );
    assert!(preemption.is_ok());
    let Ok(preemption) = preemption else {
        return;
    };
    assert_eq!(
        admit_preemption(&request, &prior, "snapshot-zya-2", &bytes, preemption),
        Err(mind_domain::AdmissionRejection::CurrentState)
    );
    assert!(FakeCheckpointPort::new().load().is_none());
}
