use mind_persistence::{BudgetProfile, Capsule, CapsuleError, CommitmentProjection, LedgerRange};

fn profile(provider: &str, model: &str, used: u32) -> BudgetProfile {
    BudgetProfile::new(provider, model, 100, used, 20)
}

fn commitments() -> CommitmentProjection {
    CommitmentProjection::new("goal", "plan", "posture", "initiative-owner")
}

#[test]
fn provider_relative_budget_controls_capsule_eligibility_and_identity() {
    let range = LedgerRange::new(4, 8, "ledger-hash");
    let exact = Capsule::new(
        range.clone(),
        profile("provider-a", "model-a", 80),
        commitments(),
        None,
    );
    let below = Capsule::new(
        range.clone(),
        profile("provider-a", "model-a", 79),
        commitments(),
        None,
    );
    let above = Capsule::new(
        range.clone(),
        profile("provider-a", "model-a", 81),
        commitments(),
        None,
    );
    let overflow = Capsule::new(
        range,
        BudgetProfile::new("provider-a", "model-a", u32::MAX, u32::MAX, 1),
        commitments(),
        None,
    );
    assert!(exact.is_ok() && below.is_ok() && above.is_ok() && overflow.is_ok());
    let (Ok(exact), Ok(below), Ok(above), Ok(overflow)) = (exact, below, above, overflow) else {
        return;
    };
    assert!(below.eligible());
    assert!(exact.eligible());
    assert!(!above.eligible());
    assert!(!overflow.eligible());
    assert_ne!(exact.identity(), below.identity());
}

#[test]
fn typed_commitments_survive_corrupt_narrative_without_authority_inversion() {
    let capsule = Capsule::new(
        LedgerRange::new(4, 8, "ledger-hash"),
        profile("provider-a", "model-a", 80),
        commitments(),
        Some("summary"),
    );
    assert!(capsule.is_ok());
    let Ok(capsule) = capsule else { return };
    let corrupted = capsule.with_narrative(Some("corrupted narrative"));
    assert_eq!(corrupted.typed_commitments(), commitments());
    assert_eq!(corrupted.narrative(), Some("corrupted narrative"));
}

#[test]
fn rejects_inconsistent_ranges_profiles_and_oversized_narrative() {
    assert_eq!(
        Capsule::new(
            LedgerRange::new(8, 4, "hash"),
            profile("p", "m", 80),
            commitments(),
            None
        ),
        Err(CapsuleError::InvalidRange)
    );
    assert_eq!(
        Capsule::new(
            LedgerRange::new(4, 8, "hash"),
            profile("p", "m", 101),
            commitments(),
            None
        ),
        Err(CapsuleError::InvalidBudget)
    );
    assert_eq!(
        Capsule::new(
            LedgerRange::new(4, 8, "hash"),
            profile("p", "m", 80),
            commitments(),
            Some(&"x".repeat(1025))
        ),
        Err(CapsuleError::NarrativeTooLarge)
    );
}
