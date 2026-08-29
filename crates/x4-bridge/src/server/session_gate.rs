use crate::{BackpressureOutcome, CapabilityDecision, SequenceNumber, SessionHello};

use super::{PipeDisposition, PipeServer};

impl PipeServer {
    pub(super) fn admit_hello(
        &mut self,
        protocol_major: u16,
        game_build: String,
        capabilities: Vec<String>,
        generation: u64,
    ) -> PipeDisposition {
        let Some(previous) = self.client_generation else {
            self.session = self.session.admit_hello(SessionHello::new(
                protocol_major,
                game_build,
                capabilities,
            ));
            self.client_generation = Some(generation);
            return hello_disposition(self.session.decision());
        };
        if generation <= previous {
            return PipeDisposition::Rejected;
        }
        self.session = self.session.reconnect();
        if self.session.decision() != CapabilityDecision::Compatible {
            return PipeDisposition::Rejected;
        }
        self.client_generation = Some(generation);
        self.discard_pending();
        PipeDisposition::Accepted
    }

    pub(super) fn admit_data(
        &mut self,
        payload: &str,
        kind: &str,
        scope: &str,
        version: u64,
        generation: u64,
        sequence: u64,
    ) -> PipeDisposition {
        if self.client_generation != Some(generation) {
            self.discard_pending();
            return PipeDisposition::Rejected;
        }
        let Some(session) = self.session.accept_sequence(SequenceNumber::new(sequence)) else {
            self.discard_pending();
            return PipeDisposition::Rejected;
        };
        if self.confirm_pending_completion() == PipeDisposition::Rejected {
            self.discard_pending();
            return PipeDisposition::Rejected;
        }
        if kind != "observation" {
            return self.admit_control(payload, kind, scope, version, session);
        }
        let (ingress, outcome) = self
            .ingress
            .submit(&session, SequenceNumber::new(sequence), kind, payload)
            .into_parts();
        if outcome != BackpressureOutcome::Accepted {
            self.discard_pending();
            return PipeDisposition::Rejected;
        }
        self.ingress = ingress;
        self.session = session;
        let disposition = self.buffer_observation(payload, scope, version);
        self.ingress = self.ingress.release();
        disposition
    }

    fn admit_control(
        &mut self,
        payload: &str,
        kind: &str,
        scope: &str,
        version: u64,
        session: crate::SessionState,
    ) -> PipeDisposition {
        self.session = session;
        match kind {
            "complete_marker" => self.defer_completion(payload, scope, version),
            "heartbeat" | "runtime_health" => self.admit_messages(&[payload]),
            _ => PipeDisposition::Rejected,
        }
    }

    fn defer_completion(&mut self, marker: &str, scope: &str, version: u64) -> PipeDisposition {
        let Some(pending) = &self.pending else {
            return PipeDisposition::Rejected;
        };
        if pending.scope != scope || pending.version != version {
            self.discard_pending();
            return PipeDisposition::Rejected;
        }
        self.pending_marker = Some(marker.to_owned());
        PipeDisposition::Accepted
    }

    fn confirm_pending_completion(&mut self) -> PipeDisposition {
        let Some(marker) = self.pending_marker.take() else {
            return PipeDisposition::Accepted;
        };
        let Some(pending) = &self.pending else {
            return PipeDisposition::Rejected;
        };
        let scope = pending.scope.clone();
        self.complete_snapshot(&marker, &scope, pending.version)
    }
}

const fn hello_disposition(decision: CapabilityDecision) -> PipeDisposition {
    match decision {
        CapabilityDecision::Compatible => PipeDisposition::Accepted,
        CapabilityDecision::RestartRequired => PipeDisposition::Rejected,
    }
}
