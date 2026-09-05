use crate::{CompletionCoverage, SectionCompletionEnvelope};

impl SectionCompletionEnvelope {
    #[must_use]
    pub fn coverage_from_wire(value: &str) -> Option<CompletionCoverage> {
        match value {
            "complete" => Some(CompletionCoverage::Complete),
            "known_empty" => Some(CompletionCoverage::KnownEmpty),
            "partial" => Some(CompletionCoverage::Partial),
            "unknown" => Some(CompletionCoverage::Unknown),
            "unsupported" => Some(CompletionCoverage::Unsupported),
            _ => None,
        }
    }

    #[must_use]
    pub const fn is_qualified_known_empty(&self) -> bool {
        self.record_count == 0 && matches!(self.coverage, CompletionCoverage::KnownEmpty)
    }
}
