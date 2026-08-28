use mind_persistence::{BudgetProfile, Capsule, CommitmentProjection, LedgerRange};

#[test]
fn rejects_budget_that_exceeds_provider_limit() {
    let profile = BudgetProfile::new("provider", "model", 10, 11, 1);
    let capsule = Capsule::new(
        LedgerRange::new(1, 2, "hash"),
        profile,
        CommitmentProjection::new("goal", "plan", "posture", "owner"),
        None,
    );
    assert!(capsule.is_err());
}

#[test]
fn keeps_typed_projection_when_narrative_changes() {
    let capsule = Capsule::new(
        LedgerRange::new(1, 2, "hash"),
        BudgetProfile::new("provider", "model", 10, 9, 1),
        CommitmentProjection::new("goal", "plan", "posture", "owner"),
        Some("narrative"),
    );
    assert!(capsule.is_ok());
    let Ok(capsule) = capsule else { return };
    assert_eq!(
        capsule.with_narrative(None).typed_commitments(),
        capsule.typed_commitments()
    );
}
