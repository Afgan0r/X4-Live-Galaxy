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
    let compact = Capsule::new(
        range.clone(),
        profile("provider-a", "model-a", 80),
        commitments(),
        None,
    );
    let retain = Capsule::new(
        range,
        profile("provider-b", "model-b", 79),
        commitments(),
        None,
    );
    assert!(compact.is_ok() && retain.is_ok());
    let (Ok(compact), Ok(retain)) = (compact, retain) else {
        return;
    };
    assert!(compact.eligible());
    assert!(!retain.eligible());
    assert_ne!(compact.identity(), retain.identity());
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
