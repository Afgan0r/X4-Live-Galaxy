use observation_ingest::{ControlEnvelope, TransportEpoch};

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
}

pub fn encode_carrier_control(
    kind: ControlEnvelope,
    identity: &CarrierIdentity,
    limit: usize,
) -> Result<Vec<u8>, CarrierCodecError> {
    validate_identity(identity)?;
    let text = format!(
        "v=1;kind={};session={};incarnation={};epoch={}",
        kind_name(kind),
        identity.session_id,
        identity.producer_incarnation,
        identity.epoch.get()
    );
    (text.len() <= limit)
        .then(|| text.into_bytes())
        .ok_or(CarrierCodecError::MessageTooLarge)
}

pub fn decode_carrier_control(
    bytes: &[u8],
    expected: &CarrierIdentity,
    limit: usize,
) -> Result<ControlEnvelope, CarrierCodecError> {
    if bytes.len() > limit {
        return Err(CarrierCodecError::MessageTooLarge);
    }
    validate_identity(expected)?;
    let text = std::str::from_utf8(bytes).map_err(|_| CarrierCodecError::InvalidShape)?;
    let fields: Vec<_> = text.split(';').collect();
    if fields.len() > 5 {
        return Err(CarrierCodecError::UnknownField);
    }
    if fields.len() != 5 {
        return Err(CarrierCodecError::InvalidShape);
    }
    let values = ["v=", "kind=", "session=", "incarnation=", "epoch="]
        .into_iter()
        .zip(fields)
        .map(|(prefix, field)| field.strip_prefix(prefix))
        .collect::<Option<Vec<_>>>()
        .ok_or(CarrierCodecError::UnknownField)?;
    if values[0] != "1" {
        return Err(CarrierCodecError::InvalidShape);
    }
    let kind = parse_kind(values[1])?;
    if values[2] != expected.session_id || values[3] != expected.producer_incarnation {
        return Err(CarrierCodecError::InvalidIdentity);
    }
    let epoch = values[4]
        .parse::<u64>()
        .map_err(|_| CarrierCodecError::InvalidIdentity)?;
    if epoch != expected.epoch.get() {
        return Err(CarrierCodecError::StaleEpoch);
    }
    Ok(kind)
}

pub fn validate_identity(identity: &CarrierIdentity) -> Result<(), CarrierCodecError> {
    let invalid = identity.session_id.trim().is_empty()
        || identity.producer_incarnation.trim().is_empty()
        || identity.session_id.contains([';', '='])
        || identity.producer_incarnation.contains([';', '=']);
    (!invalid)
        .then_some(())
        .ok_or(CarrierCodecError::InvalidIdentity)
}

const fn kind_name(kind: ControlEnvelope) -> &'static str {
    match kind {
        ControlEnvelope::Handshake => "handshake",
        ControlEnvelope::Demand => "demand",
        ControlEnvelope::Disposition => "disposition",
        ControlEnvelope::CollectionIntent => "collection_intent",
        ControlEnvelope::Health => "health",
        ControlEnvelope::Reset => "reset",
    }
}

fn parse_kind(value: &str) -> Result<ControlEnvelope, CarrierCodecError> {
    match value {
        "handshake" => Ok(ControlEnvelope::Handshake),
        "demand" => Ok(ControlEnvelope::Demand),
        "disposition" => Ok(ControlEnvelope::Disposition),
        "collection_intent" => Ok(ControlEnvelope::CollectionIntent),
        "health" => Ok(ControlEnvelope::Health),
        "reset" => Ok(ControlEnvelope::Reset),
        "mutation" | "command" | "game_action" => Err(CarrierCodecError::MutationForbidden),
        _ => Err(CarrierCodecError::UnknownKind),
    }
}
