use observation_domain::{ControlEnvelope, TransportEpoch};
use serde::{Deserialize, Serialize};

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CarrierIdentity {
    pub session_id: String,
    pub producer_incarnation: String,
    pub epoch: TransportEpoch,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CarrierCodecError {
    MessageTooLarge,
    InvalidShape,
    UnknownKind,
    UnknownField,
    InvalidIdentity,
    StaleEpoch,
    MutationForbidden,
    RestartRequired,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct CarrierControl {
    pub identity: CarrierIdentity,
    pub body: ControlBody,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ControlBody {
    Handshake(HandshakeBody),
    Demand(DemandBody),
    Disposition(DispositionBody),
    CollectionIntent(CollectionIntentBody),
    Health(HealthBody),
    Reset(ResetBody),
}

#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HandshakeBody {
    pub native_abi: u16,
    pub envelope_contract: u16,
    pub schema_version: u16,
    pub policy_version: u16,
    pub canonicalization_version: u16,
    pub digest_version: u16,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DemandBody {
    pub credit: u8,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct DispositionBody {
    pub message_id: String,
    pub section_key: String,
    pub section_revision: u64,
    pub message_digest: String,
    pub disposition: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct CollectionIntentBody {
    pub section_key: String,
    pub max_records: usize,
    pub max_raw_bytes: usize,
    pub max_work: usize,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct HealthBody {
    pub status: String,
}
#[derive(Clone, Debug, Eq, PartialEq, Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
pub struct ResetBody {
    pub reason: String,
}

#[derive(Serialize, Deserialize)]
#[serde(deny_unknown_fields)]
struct RawControl {
    control_version: u16,
    kind: String,
    session: String,
    incarnation: String,
    epoch: u64,
    body: serde_json::Value,
}

pub fn encode_carrier_control(
    control: &CarrierControl,
    limit: usize,
) -> Result<Vec<u8>, CarrierCodecError> {
    validate_identity(&control.identity)?;
    validate_body(&control.body)?;
    let (kind, body) = encode_body(&control.body)?;
    let raw = RawControl {
        control_version: 2,
        kind: kind.to_owned(),
        session: control.identity.session_id.clone(),
        incarnation: control.identity.producer_incarnation.clone(),
        epoch: control.identity.epoch.get(),
        body,
    };
    let bytes = serde_json::to_vec(&raw).map_err(|_| CarrierCodecError::InvalidShape)?;
    (bytes.len() <= limit)
        .then_some(bytes)
        .ok_or(CarrierCodecError::MessageTooLarge)
}

pub fn decode_carrier_control(
    bytes: &[u8],
    expected: &CarrierIdentity,
    limit: usize,
) -> Result<CarrierControl, CarrierCodecError> {
    if bytes.len() > limit {
        return Err(CarrierCodecError::MessageTooLarge);
    }
    validate_identity(expected)?;
    let raw: RawControl = serde_json::from_slice(bytes).map_err(|error| classify_shape(&error))?;
    if raw.control_version != 2 {
        return Err(CarrierCodecError::RestartRequired);
    }
    if raw.session != expected.session_id || raw.incarnation != expected.producer_incarnation {
        return Err(CarrierCodecError::InvalidIdentity);
    }
    if raw.epoch != expected.epoch.get() {
        return Err(CarrierCodecError::StaleEpoch);
    }
    let body = decode_body(&raw.kind, raw.body)?;
    validate_body(&body)?;
    Ok(CarrierControl {
        identity: expected.clone(),
        body,
    })
}

pub fn validate_identity(identity: &CarrierIdentity) -> Result<(), CarrierCodecError> {
    let invalid = identity.session_id.trim().is_empty()
        || identity.producer_incarnation.trim().is_empty()
        || identity.session_id.len() > 128
        || identity.producer_incarnation.len() > 128;
    (!invalid)
        .then_some(())
        .ok_or(CarrierCodecError::InvalidIdentity)
}

fn validate_body(body: &ControlBody) -> Result<(), CarrierCodecError> {
    match body {
        ControlBody::Handshake(value) if *value != exact_handshake() => {
            Err(CarrierCodecError::RestartRequired)
        }
        ControlBody::Demand(value) if value.credit != 1 => Err(CarrierCodecError::InvalidShape),
        ControlBody::Disposition(value)
            if value.message_id.is_empty()
                || value.section_key.is_empty()
                || value.section_revision == 0
                || value.message_digest.len() != 64
                || !matches!(
                    value.disposition.as_str(),
                    "received" | "committed" | "retryable_rejection" | "permanent_rejection"
                ) =>
        {
            Err(CarrierCodecError::InvalidShape)
        }
        ControlBody::CollectionIntent(value)
            if value.section_key.is_empty()
                || value.max_records == 0
                || value.max_raw_bytes == 0
                || value.max_work == 0 =>
        {
            Err(CarrierCodecError::InvalidShape)
        }
        ControlBody::Health(value) if value.status.is_empty() => {
            Err(CarrierCodecError::InvalidShape)
        }
        ControlBody::Reset(value) if value.reason.is_empty() => {
            Err(CarrierCodecError::InvalidShape)
        }
        _ => Ok(()),
    }
}

pub const fn exact_handshake() -> HandshakeBody {
    HandshakeBody {
        native_abi: 2,
        envelope_contract: 2,
        schema_version: 1,
        policy_version: 2,
        canonicalization_version: 3,
        digest_version: 1,
    }
}
pub const fn kind(body: &ControlBody) -> ControlEnvelope {
    match body {
        ControlBody::Handshake(_) => ControlEnvelope::Handshake,
        ControlBody::Demand(_) => ControlEnvelope::Demand,
        ControlBody::Disposition(_) => ControlEnvelope::Disposition,
        ControlBody::CollectionIntent(_) => ControlEnvelope::CollectionIntent,
        ControlBody::Health(_) => ControlEnvelope::Health,
        ControlBody::Reset(_) => ControlEnvelope::Reset,
    }
}
fn encode_body(body: &ControlBody) -> Result<(&'static str, serde_json::Value), CarrierCodecError> {
    let value = match body {
        ControlBody::Handshake(v) => ("handshake", serde_json::to_value(v)),
        ControlBody::Demand(v) => ("demand", serde_json::to_value(v)),
        ControlBody::Disposition(v) => ("disposition", serde_json::to_value(v)),
        ControlBody::CollectionIntent(v) => ("collection_intent", serde_json::to_value(v)),
        ControlBody::Health(v) => ("health", serde_json::to_value(v)),
        ControlBody::Reset(v) => ("reset", serde_json::to_value(v)),
    };
    Ok((
        value.0,
        value.1.map_err(|_| CarrierCodecError::InvalidShape)?,
    ))
}
fn decode_body(kind: &str, value: serde_json::Value) -> Result<ControlBody, CarrierCodecError> {
    macro_rules! body {
        ($ty:ty, $variant:ident) => {
            serde_json::from_value::<$ty>(value)
                .map(ControlBody::$variant)
                .map_err(|error| classify_shape(&error))
        };
    }
    match kind {
        "handshake" => body!(HandshakeBody, Handshake),
        "demand" => body!(DemandBody, Demand),
        "disposition" => body!(DispositionBody, Disposition),
        "collection_intent" => body!(CollectionIntentBody, CollectionIntent),
        "health" => body!(HealthBody, Health),
        "reset" => body!(ResetBody, Reset),
        "mutation" | "command" | "game_action" => Err(CarrierCodecError::MutationForbidden),
        _ => Err(CarrierCodecError::UnknownKind),
    }
}
fn classify_shape(error: &serde_json::Error) -> CarrierCodecError {
    if error.to_string().contains("unknown field") {
        CarrierCodecError::UnknownField
    } else {
        CarrierCodecError::InvalidShape
    }
}
