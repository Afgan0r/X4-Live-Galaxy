use crate::{CandidateContext, CompletionCurrent};

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ApplicationContextIdentity {
    Start(CandidateContext),
    Batch,
    Completion(CompletionCurrent),
}
