#![forbid(unsafe_code)]

mod causal;
mod mind;

pub use causal::{CausalEvent, CausalKind};
pub use mind::{CommandId, MindAggregate, MindCommand, MindError, PendingMindCommit, transition};
