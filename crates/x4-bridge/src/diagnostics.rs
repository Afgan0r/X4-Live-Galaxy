use std::path::Path;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum DiagnosticError {
    NotReady,
}

pub fn readback_revision(
    _data_dir: &Path,
    _section_key: &str,
    _section_revision: u64,
) -> Result<String, DiagnosticError> {
    Err(DiagnosticError::NotReady)
}
