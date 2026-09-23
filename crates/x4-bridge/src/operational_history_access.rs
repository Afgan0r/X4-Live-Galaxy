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
        self.revision = None;
    }

    pub fn bind_message(&mut self, message: &str, section: &str, revision: u64) {
        self.reset_duplicate_window();
        message.clone_into(&mut self.message);
        section.clone_into(&mut self.section);
        self.revision = Some(revision);
    }

    pub fn bind_selection(&mut self, attempt: u64, section: &str, revision: Option<u64>) {
        self.reset_duplicate_window();
        self.message = format!("attempt-{attempt}");
        section.clone_into(&mut self.section);
        self.revision = revision;
    }
}
