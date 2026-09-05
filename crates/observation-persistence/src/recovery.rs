use std::num::NonZeroUsize;

use crate::{PublicationReceipt, RepositoryDiagnostic};

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PublicationFailpoint {
    BeforeContent,
    AfterContent,
    AfterReceipt,
    AfterPointer,
    BeforeCommit,
    CommitResultUnknown,
    AfterCommitBeforeResponse,
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ReconciliationOutcome {
    CommittedReplay(PublicationReceipt),
    ProvenNotCommitted,
    Superseded(RepositoryDiagnostic),
    Ambiguous(RepositoryDiagnostic),
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetentionPolicy {
    pub(crate) history_per_section: NonZeroUsize,
    pub(crate) receipt_count: NonZeroUsize,
}

impl RetentionPolicy {
    #[must_use]
    pub const fn new(history_per_section: usize, receipt_count: usize) -> Option<Self> {
        Some(Self {
            history_per_section: match NonZeroUsize::new(history_per_section) {
                Some(value) => value,
                None => return None,
            },
            receipt_count: match NonZeroUsize::new(receipt_count) {
                Some(value) => value,
                None => return None,
            },
        })
    }
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct RetentionReport {
    pub deleted_revisions: usize,
}
