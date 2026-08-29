#![forbid(unsafe_code)]

mod admission;
mod causal;
mod checkpoint;
mod deliberation;
mod initiative;
mod initiative_events;
mod ledger;
mod mind;
mod restore;

pub use admission::{AcceptedProposal, AdmissionDecision, AdmissionRejection, admit};
pub use causal::{CausalEvent, CausalKind};
pub use checkpoint::{MIND_CHECKPOINT_SCHEMA_VERSION, MindCheckpointError, MindCheckpointState};
pub use deliberation::{DeliberationRequest, RequestError, ShadowProposal};
pub use initiative::{
    Initiative, InitiativeCommand, InitiativeError, InitiativeId, InitiativeLifecycle,
    InitiativeSpec, PreemptionDisposition,
};
pub use ledger::PendingInitiativeCommit;
pub use mind::{CommandId, MindAggregate, MindCommand, MindError, PendingMindCommit, transition};
pub use strategic_state::Capability;
