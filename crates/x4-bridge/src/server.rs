use crate::{BoundedIngress, FrameLimits, SessionState};
use observation_ingest::{
    AcceptedProjection, AdmissionOutcome, FrameHeader, MAX_BATCH_BYTES, ProjectionSnapshot,
    admit_batch, inspect_frame,
};

pub const PIPE_ENDPOINT: &str = r"\\.\pipe\live_galaxy";

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum PipeDisposition {
    Accepted,
    Rejected,
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcceptDisposition {
    ServeClient,
    RetryAccept,
    RetryAcceptDegraded { delay_millis: u64 },
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum AcceptAttempt {
    ClientAccepted,
    TransientFailure,
}

pub const MAX_CONSECUTIVE_ACCEPT_FAILURES: usize = 3;
pub const MAX_ACCEPT_DELAY_MILLIS: u64 = 1_000;
const INITIAL_ACCEPT_DELAY_MILLIS: u64 = 100;
const MAX_PENDING_OBSERVATIONS: usize = 64;
const EXPECTED_PROTOCOL_MAJOR: u16 = 1;
const MAX_PIPE_FRAME_BYTES: usize = 2_048;

mod session_gate;

#[derive(Clone, Debug)]
struct PendingSnapshot {
    scope: String,
    version: u64,
    bytes: usize,
    observations: Vec<String>,
}

#[derive(Clone, Debug)]
pub struct PipeServer {
    projection: AcceptedProjection,
    consecutive_accept_failures: usize,
    accept_delay_millis: u64,
    pending: Option<PendingSnapshot>,
    session: SessionState,
    ingress: BoundedIngress,
    client_generation: Option<u64>,
}

impl PipeServer {
    #[must_use]
    pub fn new() -> Self {
        Self::default()
    }

    pub fn admit_message(&mut self, payload: &str) -> PipeDisposition {
        match inspect_frame(payload) {
            Ok(FrameHeader::Hello {
                protocol_major,
                game_build,
                capabilities,
                generation,
            }) => self.admit_hello(protocol_major, game_build, capabilities, generation),
            Ok(FrameHeader::Data {
                kind,
                scope,
                version,
                generation,
                sequence,
            }) => self.admit_data(payload, kind, &scope, version, generation, sequence),
            Err(_) => {
                self.discard_pending();
                PipeDisposition::Rejected
            }
        }
    }

    pub fn admit_messages(&mut self, payloads: &[&str]) -> PipeDisposition {
        let outcome = admit_batch(self.projection.clone(), payloads);
        self.projection = outcome.projection().clone();
        match outcome {
            AdmissionOutcome::Accepted(_) => PipeDisposition::Accepted,
            AdmissionOutcome::Rejected { .. } => PipeDisposition::Rejected,
        }
    }

    pub const fn snapshot(&self) -> &ProjectionSnapshot {
        self.projection.snapshot()
    }

    pub fn discard_pending(&mut self) {
        self.pending = None;
    }

    pub fn record_accept(&mut self, attempt: AcceptAttempt) -> AcceptDisposition {
        if attempt == AcceptAttempt::ClientAccepted {
            self.consecutive_accept_failures = 0;
            self.accept_delay_millis = 0;
            return AcceptDisposition::ServeClient;
        }
        self.consecutive_accept_failures =
            (self.consecutive_accept_failures + 1).min(MAX_CONSECUTIVE_ACCEPT_FAILURES);
        if self.consecutive_accept_failures < MAX_CONSECUTIVE_ACCEPT_FAILURES {
            AcceptDisposition::RetryAccept
        } else {
            self.accept_delay_millis = next_accept_delay(self.accept_delay_millis);
            AcceptDisposition::RetryAcceptDegraded {
                delay_millis: self.accept_delay_millis,
            }
        }
    }

    fn buffer_observation(&mut self, payload: &str, scope: &str, version: u64) -> PipeDisposition {
        let pending = self.pending.get_or_insert_with(|| PendingSnapshot {
            scope: scope.to_owned(),
            version,
            bytes: 0,
            observations: Vec::new(),
        });
        if pending.scope != scope
            || pending.version != version
            || pending.observations.len() == MAX_PENDING_OBSERVATIONS
            || pending.bytes + payload.len() > MAX_BATCH_BYTES - MAX_PIPE_FRAME_BYTES
        {
            self.discard_pending();
            return PipeDisposition::Rejected;
        }
        pending.bytes += payload.len();
        pending.observations.push(payload.to_owned());
        PipeDisposition::Accepted
    }

    fn complete_snapshot(&mut self, marker: &str, scope: &str, version: u64) -> PipeDisposition {
        let Some(pending) = self.pending.take() else {
            return PipeDisposition::Rejected;
        };
        if pending.scope != scope || pending.version != version {
            return PipeDisposition::Rejected;
        }
        let mut frames = pending
            .observations
            .iter()
            .map(String::as_str)
            .collect::<Vec<_>>();
        frames.push(marker);
        self.admit_messages(&frames)
    }
}

impl Default for PipeServer {
    fn default() -> Self {
        Self {
            projection: AcceptedProjection::default(),
            consecutive_accept_failures: 0,
            accept_delay_millis: 0,
            pending: None,
            session: SessionState::new(EXPECTED_PROTOCOL_MAJOR),
            ingress: BoundedIngress::new(FrameLimits::new(
                MAX_PIPE_FRAME_BYTES,
                MAX_PENDING_OBSERVATIONS,
            )),
            client_generation: None,
        }
    }
}

fn next_accept_delay(previous: u64) -> u64 {
    if previous == 0 {
        INITIAL_ACCEPT_DELAY_MILLIS
    } else {
        (previous * 2).min(MAX_ACCEPT_DELAY_MILLIS)
    }
}
