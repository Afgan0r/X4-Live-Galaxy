use std::{
    fs::{File, OpenOptions},
    io::Write as _,
    path::{Path, PathBuf},
    time::{SystemTime, UNIX_EPOCH},
};

use crate::{DiagnosticError, diagnostics::escape};

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
        let sink = open_append(&path)?;
        let mut history = Self {
            sink: Some(sink),
            path,
            written,
            last: None,
            repeated: 0,
            suppressed: 0,
            gaps: 0,
            emergency_reported: false,
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
        if self.is_suppressed(state, reason) {
            self.suppressed = self.suppressed.saturating_add(1);
            return self.write_status(state, reason);
        }
        let line = self.event_line(state, reason);
        self.suppressed = 0;
        if self.written.saturating_add(line.len()) > MAX_HISTORY_BYTES && !self.rotate() {
            return self.note_gap(state, reason);
        }
        let accepted = self
            .sink
            .as_mut()
            .is_some_and(|sink| sink.write_all(line.as_bytes()).is_ok() && sink.flush().is_ok());
        if !accepted {
            return self.note_gap(state, reason);
        }
        self.written = self.written.saturating_add(line.len());
        self.write_status(state, reason)
    }

    #[must_use]
    pub const fn has_history_gap(&self) -> bool {
        self.gaps > 0
    }

    #[must_use]
    pub const fn history_gap_count(&self) -> u64 {
        self.gaps
    }

    pub fn bind_session(&mut self, session: &str, epoch: u64) {
        session.clone_into(&mut self.session);
        self.epoch = epoch;
        self.message.clear();
        self.section.clear();
        self.revision = 0;
    }

    pub fn bind_message(&mut self, message: &str, section: &str, revision: u64) {
        message.clone_into(&mut self.message);
        section.clone_into(&mut self.section);
        self.revision = revision;
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

    fn event_line(&self, state: &str, reason: &str) -> String {
        format!(
            "{{\"clock\":\"unix-ms\",\"at\":{},\"component\":\"x4-bridge\",\"session\":\"{}\",\"epoch\":{},\"message\":\"{}\",\"section\":\"{}\",\"revision\":{},\"state\":\"{}\",\"reason\":\"{}\",\"suppressed_before\":{}}}\n",
            now(),
            escape(&self.session),
            self.epoch,
            escape(&self.message),
            escape(&self.section),
            self.revision,
            escape(state),
            escape(reason),
            self.suppressed
        )
    }

    fn rotate(&mut self) -> bool {
        self.sink = None;
        let retained = self.path.with_extension("jsonl.1");
        if retained.exists() && std::fs::remove_file(&retained).is_err() {
            return false;
        }
        if self.path.exists() && std::fs::rename(&self.path, retained).is_err() {
            return false;
        }
        let Ok(sink) = open_append(&self.path) else {
            return false;
        };
        self.sink = Some(sink);
        self.written = 0;
        true
    }

    fn note_gap(&mut self, state: &str, reason: &str) -> bool {
        self.sink = None;
        self.gaps = self.gaps.saturating_add(1);
        if !self.emergency_reported {
            let _ = std::io::stderr().write_all(b"x4-bridge degraded: operational-history-gap\n");
            self.emergency_reported = true;
        }
        let _ = self.write_status(state, reason);
        false
    }

    fn write_status(&self, state: &str, reason: &str) -> bool {
        let path = self.path.with_file_name("operational-status.json");
        let value = format!(
            "{{\"component\":\"x4-bridge\",\"state\":\"{}\",\"reason\":\"{}\",\"history_gap_count\":{},\"suppressed_count\":{}}}\n",
            escape(state),
            escape(reason),
            self.gaps,
            self.suppressed
        );
        std::fs::write(path, value).is_ok()
    }
}

fn open_append(path: &Path) -> Result<File, DiagnosticError> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|_| DiagnosticError::Storage)
}

fn now() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .map_or(0, |value| {
            u64::try_from(value.as_millis()).unwrap_or(u64::MAX)
        })
}

#[cfg(test)]
#[path = "operational_history_tests.rs"]
mod tests;
