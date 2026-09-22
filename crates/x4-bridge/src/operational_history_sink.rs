use std::{
    fs::{File, OpenOptions},
    io::Write as _,
    path::Path,
};

use crate::DiagnosticError;

pub(super) fn open_append(path: &Path) -> Result<File, DiagnosticError> {
    OpenOptions::new()
        .create(true)
        .append(true)
        .open(path)
        .map_err(|_| DiagnosticError::Storage)
}

pub(super) fn write_line(sink: Option<&mut File>, line: &str) -> bool {
    sink.is_some_and(|value| value.write_all(line.as_bytes()).is_ok() && value.flush().is_ok())
}

pub(super) fn rotate(path: &Path, sink: &mut Option<File>, written: &mut usize) -> bool {
    *sink = None;
    let retained = path.with_extension("jsonl.1");
    if retained.exists() && std::fs::remove_file(&retained).is_err() {
        return false;
    }
    if path.exists() && std::fs::rename(path, retained).is_err() {
        return false;
    }
    let Ok(replacement) = open_append(path) else {
        return false;
    };
    *sink = Some(replacement);
    *written = 0;
    true
}
