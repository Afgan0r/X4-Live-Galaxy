#![forbid(unsafe_code)]

mod ingress;
mod protocol;
mod session;
mod telemetry;

pub use ingress::{BackpressureOutcome, BoundedIngress, FrameLimits, IngressSubmission};
pub use protocol::{CapabilityDecision, RestartRequirement, SessionHello};
pub use session::{SequenceNumber, SessionGeneration, SessionState};
pub use telemetry::{BridgeError, TelemetryFrame, admit_tracer_frame};
