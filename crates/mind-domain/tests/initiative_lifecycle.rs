use mind_domain::{
    Capability, CommandId, InitiativeCommand, InitiativeId, InitiativeSpec, MindAggregate,
    PreemptionDisposition,
};
use strategic_state::Faction;

const fn defense(id: &'static str) -> InitiativeSpec {
    InitiativeSpec::new(
        InitiativeId::new(id),
        Capability::DefenseAndMilitaryStrategy,
        "defend frontier",
        "military-fact",
        90,
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
