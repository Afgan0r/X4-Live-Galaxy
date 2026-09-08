use observation_ingest::{CarrierControl, CarrierIdentity, ControlBody, encode_carrier_control};

use crate::producer_message::SectionMessages;
use crate::{
    ProducerError, ProducerLimits, ProducerSource, ProducerState, SectionEvidence, TypedFact,
};

pub(super) struct Pending {
    pub(super) bytes: Vec<u8>,
    pub(super) id: String,
    pub(super) handed_off: bool,
    pub(super) attempts: u8,
    pub(super) first_attempt_at: u64,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum Readiness {
    Awaiting,
    Handshake,
    Intent,
    Ready,
}

pub struct Producer {
    pub(super) limits: ProducerLimits,
    pub(super) source: ProducerSource,
    pub(super) state: ProducerState,
    pub(super) readiness: Readiness,
    pub(super) revision: u64,
    pub(super) evidence: Option<SectionEvidence>,
    pub(super) fact: Option<TypedFact>,
    pub(super) finished: bool,
    pub(super) messages: Option<SectionMessages>,
    pub(super) pending: Option<Pending>,
    pub(super) connection_generation: u64,
}

impl Producer {
    pub fn new(
        limits: ProducerLimits,
        source: ProducerSource,
        now_millis: u64,
    ) -> Result<Self, ProducerError> {
        if !limits.valid() {
            return Err(ProducerError::InvalidInput);
        }
        let bytes = bootstrap_bytes(&source, limits.control_message_bytes)?;
        Ok(Self {
            limits,
            source,
            state: ProducerState::AwaitingCompatibility,
            readiness: Readiness::Awaiting,
            revision: 1,
            evidence: None,
            fact: None,
            finished: false,
            messages: None,
            pending: Some(Pending::new(bytes, "bootstrap", now_millis)),
            connection_generation: 0,
        })
    }

    #[must_use]
    pub const fn state(&self) -> ProducerState {
        self.state
    }

    pub(crate) const fn source(&self) -> &ProducerSource {
        &self.source
    }

    #[must_use]
    pub fn pending_bytes(&self) -> Option<&[u8]> {
        self.pending
            .as_ref()
            .filter(|value| !value.handed_off)
            .map(|value| value.bytes.as_slice())
    }

    pub fn mark_local_handoff(&mut self, now_millis: u64) -> Result<(), ProducerError> {
        let pending = self
            .pending
            .as_mut()
            .ok_or(ProducerError::InvalidTransition)?;
        if pending.handed_off {
            return Err(ProducerError::InvalidTransition);
        }
        if pending.attempts == 1 {
            pending.first_attempt_at = now_millis;
        }
        pending.handed_off = true;
        Ok(())
    }

    pub fn fail_section(&mut self) {
        self.discard_incomplete();
        self.state = ProducerState::PausedAfterFailure;
    }

    pub fn close(&mut self) {
        self.discard_incomplete();
        self.state = ProducerState::Closed;
    }

    pub fn reset(&mut self, source: ProducerSource, now_millis: u64) -> Result<(), ProducerError> {
        *self = Self::new(self.limits, source, now_millis)?;
        Ok(())
    }

    pub(super) fn discard_incomplete(&mut self) {
        self.evidence = None;
        self.fact = None;
        self.finished = false;
        self.messages = None;
        self.pending = None;
    }
}

impl Pending {
    pub(super) fn new(bytes: Vec<u8>, id: impl Into<String>, now: u64) -> Self {
        Self {
            bytes,
            id: id.into(),
            handed_off: false,
            attempts: 1,
            first_attempt_at: now,
        }
    }
}

pub(super) fn identity(source: &ProducerSource) -> Result<CarrierIdentity, ProducerError> {
    let identity = CarrierIdentity {
        session_id: source.session_id.clone(),
        producer_incarnation: source.producer_incarnation.clone(),
        epoch: observation_domain::TransportEpoch::new(source.transport_epoch)
            .ok_or(ProducerError::StaleEpoch)?,
    };
    observation_ingest::validate_carrier_identity(&identity)
        .map_err(|_| ProducerError::InvalidInput)?;
    Ok(identity)
}

pub(super) fn bootstrap_bytes(
    source: &ProducerSource,
    limit: usize,
) -> Result<Vec<u8>, ProducerError> {
    encode_carrier_control(
        &CarrierControl {
            identity: identity(source)?,
            body: ControlBody::Handshake(observation_ingest::HandshakeBody {
                native_abi: 2,
                envelope_contract: 2,
                schema_version: 1,
                policy_version: 2,
                canonicalization_version: 3,
                digest_version: 1,
            }),
        },
        limit,
    )
    .map_err(|_| ProducerError::ControlLimit)
}
