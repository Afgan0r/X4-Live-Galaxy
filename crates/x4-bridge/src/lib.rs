#![forbid(unsafe_code)]

mod ingress;
mod listener;
mod protocol;
mod server;
mod session;
mod telemetry;

pub use ingress::{BackpressureOutcome, BoundedIngress, FrameLimits, IngressSubmission};
pub use listener::run_windows_listener;
pub use protocol::{CapabilityDecision, RestartRequirement, SessionHello};
pub use server::{
    AcceptAttempt, AcceptDisposition, MAX_ACCEPT_DELAY_MILLIS, MAX_CONSECUTIVE_ACCEPT_FAILURES,
    PIPE_ENDPOINT, PipeDisposition, PipeServer,
};
pub use session::{SequenceNumber, SessionGeneration, SessionState};
pub use telemetry::{BridgeError, TelemetryFrame, admit_tracer_frame};

#[must_use]
pub const fn is_telemetry_only() -> bool {
    true
}
