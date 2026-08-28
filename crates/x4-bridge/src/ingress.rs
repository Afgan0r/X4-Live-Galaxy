use crate::{CapabilityDecision, SequenceNumber, SessionGeneration, SessionState};

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct FrameLimits {
    max_frame_bytes: usize,
    max_queue_depth: usize,
}

impl FrameLimits {
    pub const fn new(max_frame_bytes: usize, max_queue_depth: usize) -> Self {
        Self {
            max_frame_bytes,
            max_queue_depth,
        }
    }
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum BackpressureOutcome {
    Accepted,
    FrameTooLarge,
    QueueSaturated,
    UnsupportedFrameKind,
    SessionNotCompatible,
    StaleGeneration,
    StaleSequence,
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct BoundedIngress {
    limits: FrameLimits,
    queued_frames: usize,
    session_generation: Option<SessionGeneration>,
    last_sequence: Option<SequenceNumber>,
}

impl BoundedIngress {
    pub const fn new(limits: FrameLimits) -> Self {
        Self {
            limits,
            queued_frames: 0,
            session_generation: None,
            last_sequence: None,
        }
    }

    pub fn submit(
        self,
        session: &SessionState,
        sequence: SequenceNumber,
        kind: &str,
        payload: &str,
    ) -> IngressSubmission {
        if session.decision() != CapabilityDecision::Compatible {
            return IngressSubmission::rejected(self, BackpressureOutcome::SessionNotCompatible);
        }
        let ingress = match self.bind_generation(session.generation()) {
            Ok(ingress) => ingress,
            Err(outcome) => return IngressSubmission::rejected(self, outcome),
        };
        if ingress.last_sequence.is_some_and(|last| sequence <= last) {
            return IngressSubmission::rejected(ingress, BackpressureOutcome::StaleSequence);
        }
        if kind != "observation" {
            return IngressSubmission::rejected(ingress, BackpressureOutcome::UnsupportedFrameKind);
        }
        if payload.len() > ingress.limits.max_frame_bytes {
            return IngressSubmission::rejected(ingress, BackpressureOutcome::FrameTooLarge);
        }
        if ingress.queued_frames >= ingress.limits.max_queue_depth {
            return IngressSubmission::rejected(ingress, BackpressureOutcome::QueueSaturated);
        }

        IngressSubmission::accepted(Self {
            queued_frames: ingress.queued_frames + 1,
            last_sequence: Some(sequence),
            ..ingress
        })
    }

    fn bind_generation(&self, generation: SessionGeneration) -> Result<Self, BackpressureOutcome> {
        if self
            .session_generation
            .is_some_and(|bound| generation < bound)
        {
            return Err(BackpressureOutcome::StaleGeneration);
        }
        let mut ingress = *self;
        if ingress.session_generation != Some(generation) {
            ingress.session_generation = Some(generation);
            ingress.last_sequence = None;
        }
        Ok(ingress)
    }
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct IngressSubmission {
    ingress: BoundedIngress,
    outcome: BackpressureOutcome,
}

impl IngressSubmission {
    const fn accepted(ingress: BoundedIngress) -> Self {
        Self {
            ingress,
            outcome: BackpressureOutcome::Accepted,
        }
    }

    const fn rejected(ingress: BoundedIngress, outcome: BackpressureOutcome) -> Self {
        Self { ingress, outcome }
    }

    pub const fn into_parts(self) -> (BoundedIngress, BackpressureOutcome) {
        (self.ingress, self.outcome)
    }
}
