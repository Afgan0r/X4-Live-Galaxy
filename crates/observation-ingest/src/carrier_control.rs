use serde::{Deserialize, Serialize};

use crate::carrier_control_types::{
    CarrierCodecError, CarrierControl, CarrierIdentity, CollectionIntentBody, ControlBody,
    DemandBody, DispositionBody, HandshakeBody, HealthBody, ResetBody, validate_carrier_identity,
};

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
    validate_carrier_identity(&control.identity)?;
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
    validate_carrier_identity(expected)?;
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
                    "capacity_unavailable"
                        | "received"
                        | "committed"
                        | "timed_out_or_superseded"
                        | "stale_epoch"
                        | "permanently_rejected"
                        | "ambiguous_commit"
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
