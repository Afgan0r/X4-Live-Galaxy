use super::OperationalHistory;

impl OperationalHistory {
    #[must_use]
    pub const fn history_gap_count(&self) -> u64 {
        self.gaps
    }

    pub fn bind_session(&mut self, session: &str, epoch: u64) {
        self.reset_duplicate_window();
        session.clone_into(&mut self.session);
        self.epoch = epoch;
        self.message.clear();
        self.section.clear();
        self.revision = 0;
    }

    pub fn bind_message(&mut self, message: &str, section: &str, revision: u64) {
        self.reset_duplicate_window();
        message.clone_into(&mut self.message);
        section.clone_into(&mut self.section);
        self.revision = revision;
    }
}
