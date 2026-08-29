use mind_domain::{
    Capability, CommandId, InitiativeCommand, InitiativeId, InitiativeSpec, MindAggregate,
    PreemptionDisposition,
};
use strategic_state::Faction;

fn defense(id: &str) -> InitiativeSpec {
    InitiativeSpec::new(
        InitiativeId::new(id),
        Capability::DefenseAndMilitaryStrategy,
        "defend frontier",
        "military-fact",
        90,
    )
}

fn initiative(id: &str, capability: Capability) -> InitiativeSpec {
    InitiativeSpec::new(
        InitiativeId::new(id),
        capability,
        "bounded objective",
        "supporting fact",
        50,
    )
}

#[test]
fn retains_single_owner_preemption_and_causal_history() {
    let initial = MindAggregate::empty(Faction::Zya);
    let first = initial.apply_initiative(InitiativeCommand::accept(
        CommandId::new("initiative-1"),
        defense("initiative-a"),
    ));
    assert!(first.is_ok());
    let Ok(first) = first else { return };
    assert!(
        first
            .aggregate()
            .active_initiative(Capability::DefenseAndMilitaryStrategy)
            .is_some()
    );
    assert_eq!(first.events().len(), 5);

    let duplicate = first
        .aggregate()
        .apply_initiative(InitiativeCommand::accept(
            CommandId::new("initiative-2"),
            defense("initiative-b"),
        ));
    assert!(duplicate.is_err());

    let replacement = first
        .aggregate()
        .apply_initiative(InitiativeCommand::preempt(
            CommandId::new("initiative-3"),
            InitiativeId::new("initiative-a"),
            defense("initiative-b"),
            "new threat evidence",
            PreemptionDisposition::Cancelled,
        ));
    assert!(replacement.is_ok());
    let Ok(replacement) = replacement else { return };
    assert_eq!(replacement.events().len(), 6);
    assert_eq!(replacement.aggregate().initiative_history().len(), 2);

    let retry = replacement
        .aggregate()
        .apply_initiative(InitiativeCommand::preempt(
            CommandId::new("initiative-3"),
            InitiativeId::new("initiative-a"),
            defense("initiative-b"),
            "new threat evidence",
            PreemptionDisposition::Cancelled,
        ));
    assert_eq!(Ok(replacement.clone()), retry);
    let collision = replacement
        .aggregate()
        .apply_initiative(InitiativeCommand::preempt(
            CommandId::new("initiative-3"),
            InitiativeId::new("initiative-a"),
            defense("initiative-c"),
            "new threat evidence",
            PreemptionDisposition::Cancelled,
        ));
    assert!(collision.is_err());
}

#[test]
fn terminal_outcomes_are_replayable_without_mutation() {
    for command in [
        InitiativeCommand::complete(
            CommandId::new("complete"),
            InitiativeId::new("initiative-a"),
        ),
        InitiativeCommand::cancel(CommandId::new("cancel"), InitiativeId::new("initiative-a")),
        InitiativeCommand::reject(CommandId::new("reject"), InitiativeId::new("initiative-a")),
        InitiativeCommand::fail(CommandId::new("fail"), InitiativeId::new("initiative-a")),
    ] {
        let accepted = MindAggregate::empty(Faction::Arg).apply_initiative(
            InitiativeCommand::accept(CommandId::new("accept"), defense("initiative-a")),
        );
        assert!(accepted.is_ok());
        let Ok(accepted) = accepted else { continue };
        let terminal = accepted.aggregate().apply_initiative(command);
        assert!(terminal.is_ok());
        let Ok(terminal) = terminal else { continue };
        assert!(
            terminal
                .aggregate()
                .active_initiative(Capability::DefenseAndMilitaryStrategy)
                .is_none()
        );
        assert_eq!(terminal.events().len(), 1);
    }
}

#[test]
fn exposes_exactly_three_independent_capability_slots_per_faction() {
    let capabilities = [
        Capability::DefenseAndMilitaryStrategy,
        Capability::EconomyAndLogistics,
        Capability::TerritorialDevelopmentAndInfrastructure,
    ];
    let ids = ["defense", "economy", "territorial"];
    let command_ids = ["accept-defense", "accept-economy", "accept-territorial"];
    let mut aggregate = MindAggregate::empty(Faction::Arg);

    for ((capability, id), command_id) in capabilities.into_iter().zip(ids).zip(command_ids) {
        let accepted = aggregate.apply_initiative(InitiativeCommand::accept(
            CommandId::new(command_id),
            initiative(id, capability),
        ));
        assert!(accepted.is_ok());
        let Ok(commit) = accepted else { return };
        aggregate = commit.aggregate().clone();
    }

    for capability in capabilities {
        assert!(aggregate.active_initiative(capability).is_some());
    }
    for (capability, id, command_id) in [
        (
            Capability::DefenseAndMilitaryStrategy,
            "defense-replacement",
            "duplicate-defense",
        ),
        (
            Capability::EconomyAndLogistics,
            "economy-replacement",
            "duplicate-economy",
        ),
        (
            Capability::TerritorialDevelopmentAndInfrastructure,
            "territorial-replacement",
            "duplicate-territorial",
        ),
    ] {
        assert!(
            aggregate
                .apply_initiative(InitiativeCommand::accept(
                    CommandId::new(command_id),
                    initiative(id, capability),
                ))
                .is_err()
        );
    }
}
