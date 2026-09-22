use std::fs::File;
use std::io::Write as _;
use std::path::{Path, PathBuf};

use crate::{DiagnosticError, diagnostics::escape, operational_status};
#[path = "operational_history_access.rs"]
mod access;
#[path = "operational_correlation.rs"]
mod correlation;
#[path = "operational_history_sink.rs"]
mod sink_io;
const MAX_HISTORY_BYTES: usize = 64 * 1_024;
const MAX_IDENTICAL_EVENTS: usize = 16;

pub struct OperationalHistory {
    sink: Option<File>,
    path: PathBuf,
    written: usize,
    last: Option<(String, String)>,
    repeated: usize,
    suppressed: u64,
    gaps: u64,
    emergency_reported: bool,
    status_gaps: u64,
    status_emergency_reported: bool,
    session: String,
    epoch: u64,
    message: String,
    section: String,
    revision: u64,
}

impl OperationalHistory {
    pub fn open(data_dir: &Path) -> Result<Self, DiagnosticError> {
        std::fs::create_dir_all(data_dir).map_err(|_| DiagnosticError::Storage)?;
        let path = data_dir.join("operational-history.jsonl");
        let written = std::fs::metadata(&path).map_or(0, |value| {
            usize::try_from(value.len()).unwrap_or(usize::MAX)
        });
        let sink = sink_io::open_append(&path)?;
        let mut history = Self {
            sink: Some(sink),
            path,
            written,
            last: None,
            repeated: 0,
            suppressed: 0,
            gaps: 0,
            emergency_reported: false,
            status_gaps: 0,
            status_emergency_reported: false,
            session: String::new(),
            epoch: 0,
            message: String::new(),
            section: String::new(),
            revision: 0,
        };
        history
            .record("startup", "journal-accepted")
            .then_some(history)
            .ok_or(DiagnosticError::Storage)
    }

    #[must_use]
    pub fn record(&mut self, state: &str, reason: &str) -> bool {
        let recovered = self.reopen_sink();
        if self.sink.is_none() {
            return self.note_gap(state, reason);
        }
        if recovered && !self.write_recovery() {
            return self.note_gap(state, reason);
        }
        if self.is_suppressed(state, reason) {
            self.suppressed = self.suppressed.saturating_add(1);
            return self.record_status(state, reason);
        }
        let line = self.event_line(state, reason);
        self.suppressed = 0;
        if self.written.saturating_add(line.len()) > MAX_HISTORY_BYTES && !self.rotate() {
            return self.note_gap(state, reason);
        }
        if !self.write_line(&line) {
            return self.note_gap(state, reason);
        }
        self.record_status(state, reason)
    }

    fn is_suppressed(&mut self, state: &str, reason: &str) -> bool {
        let same = self
            .last
            .as_ref()
            .is_some_and(|value| value.0 == state && value.1 == reason);
        if same {
            self.repeated = self.repeated.saturating_add(1);
        } else {
            self.last = Some((state.to_owned(), reason.to_owned()));
            self.repeated = 1;
        }
        self.repeated > MAX_IDENTICAL_EVENTS
    }

    fn reset_duplicate_window(&mut self) {
        (self.last, self.repeated) = (None, 0);
    }
    fn reopen_sink(&mut self) -> bool {
        if self.sink.is_some() {
            return false;
        }
        if let Ok(sink) = sink_io::open_append(&self.path) {
            self.sink = Some(sink);
        }
        self.sink.is_some()
    }
    fn write_line(&mut self, line: &str) -> bool {
        let accepted = sink_io::write_line(self.sink.as_mut(), line);
        if accepted {
            self.written = self.written.saturating_add(line.len());
        }
        accepted
    }
    fn write_recovery(&mut self) -> bool {
        let marker = self.event_line("journal-recovered", "append-reopened");
        let accepted = self.write_line(&marker);
        self.emergency_reported = !accepted;
        accepted
    }
    fn event_line(&self, state: &str, reason: &str) -> String {
        format!(
            "{{\"clock\":\"unix-ms\",\"at\":{},\"component\":\"x4-bridge\",\"session\":\"{}\",\"epoch\":{},\"message\":\"{}\",\"section\":\"{}\",\"scope\":\"{}\",\"revision\":{},\"attempt\":{},\"outcome\":\"{}\",\"state\":\"{}\",\"reason\":\"{}\",\"suppressed_before\":{},\"status_gap_count\":{}}}\n",
            operational_status::now(),
            escape(&self.session),
            self.epoch,
            escape(&self.message),
            escape(&self.section),
            escape(&correlation::scope(&self.section)),
            self.revision,
            correlation::attempt(&self.message),
            escape(state),
            escape(state),
            escape(reason),
            self.suppressed,
            self.status_gaps
        )
    }

    fn rotate(&mut self) -> bool {
        sink_io::rotate(&self.path, &mut self.sink, &mut self.written)
    }

    fn note_gap(&mut self, state: &str, reason: &str) -> bool {
        self.sink = None;
        self.gaps = self.gaps.saturating_add(1);
        if !self.emergency_reported {
            let _ = std::io::stderr().write_all(b"x4-bridge degraded: operational-history-gap\n");
            self.emergency_reported = true;
        }
        let _ = self.record_status(state, reason);
        false
    }

    fn record_status(&mut self, state: &str, reason: &str) -> bool {
        if operational_status::write(
            &self.path,
            state,
            reason,
            self.gaps,
            self.suppressed,
            self.status_gaps,
        ) {
            self.status_emergency_reported = false;
            return true;
        }
        self.status_gaps = self.status_gaps.saturating_add(1);
        if !self.status_emergency_reported {
            let _ = std::io::stderr().write_all(b"x4-bridge degraded: operational-status-gap\n");
            self.status_emergency_reported = true;
        }
        false
    }
}

#[cfg(test)]
#[path = "operational_history_tests.rs"]
mod tests;
