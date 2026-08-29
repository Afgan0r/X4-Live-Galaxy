use crate::{InitiativeCommand, MindAggregate};

impl MindAggregate {
    #[must_use]
    pub fn has_initiative_command(&self, command: &InitiativeCommand) -> bool {
        self.commands
            .iter()
            .any(|(recorded, _)| recorded == command)
    }
}
