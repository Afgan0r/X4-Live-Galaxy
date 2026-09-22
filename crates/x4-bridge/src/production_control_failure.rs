#[derive(Clone, Copy)]
pub(super) enum SelectionFailure {
    Cursor,
    Admission,
    Revision,
    Intent,
    Demand,
}

impl SelectionFailure {
    pub(super) const fn reason(self) -> &'static str {
        match self {
            Self::Cursor => "next-selection-cursor",
            Self::Admission => "next-selection-admission",
            Self::Revision => "next-selection-revision",
            Self::Intent => "next-selection-intent-send",
            Self::Demand => "next-selection-demand-send",
        }
    }
}
