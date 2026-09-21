use crate::ProductionError;
use observation_application::LifecycleError;
use observation_domain::{
    CompleteMessage, SenderEvidence, ShipClass, ShipIdentity, ShipLocation, ShipOwner,
    ShipRecordConsistency, ShipStaleReason, ShipType, SourceScopeId,
};

const fn rejected() -> ProductionError {
    ProductionError::Lifecycle(LifecycleError::AuthorityRejected)
}

pub fn scope(faction: &str) -> Result<SourceScopeId, ProductionError> {
    if !token(faction) || matches!(faction, "player" | "xenon" | "khaak") {
        return Err(rejected());
    }
    SourceScopeId::new(format!("x4:faction:{faction}:ships")).ok_or_else(rejected)
}

pub fn validate(
    message: &CompleteMessage,
    selected: Option<&SourceScopeId>,
) -> Result<(), ProductionError> {
    let (key, source) = match message {
        CompleteMessage::SectionStart(v) => (&v.section_key, &v.source_scope),
        CompleteMessage::ImmutableBatch(v) => (&v.section_key, &v.source_scope),
        CompleteMessage::SectionCompletion(v) => (&v.section_key, &v.source_scope),
        CompleteMessage::Control(_) => return Ok(()),
    };
    let detail = crate::receiver_ship_detail::is_key(key.as_str());
    if selected.is_some() && ((key.as_str() != "ship_core" && !detail) || selected != Some(source))
    {
        return Err(rejected());
    }
    if key.as_str() != "ship_core" && !detail {
        return Ok(());
    }
    if selected != Some(source) {
        return Err(rejected());
    }
    let faction = source
        .as_str()
        .strip_prefix("x4:faction:")
        .and_then(|v| v.strip_suffix(":ships"))
        .ok_or_else(rejected)?;
    if scope(faction)?.as_str() != source.as_str() {
        return Err(rejected());
    }
    match message {
        CompleteMessage::SectionStart(v) => {
            validate_evidence(&v.sender_evidence)?;
            if v.expected_records == 0 {
                return Err(rejected());
            }
        }
        CompleteMessage::ImmutableBatch(v) => {
            if detail {
                crate::receiver_ship_detail::validate_batch(v, faction)?;
            } else {
                validate_batch(v, faction)?;
            }
        }
        CompleteMessage::SectionCompletion(v) => {
            validate_evidence(&v.sender_evidence)?;
            if v.coverage != observation_domain::CompletionCoverage::Partial || v.record_count == 0
            {
                return Err(rejected());
            }
        }
        CompleteMessage::Control(_) => {}
    }
    Ok(())
}

pub fn validate_evidence(e: &SenderEvidence) -> Result<(), ProductionError> {
    use observation_domain::{
        SectionAvailability, SectionCoverage, SectionFreshness, SectionQuality, SourceConsistency,
    };
    let versions = SenderEvidence::legacy_default();
    if e.schema_version != versions.schema_version
        || e.policy_version != versions.policy_version
        || e.canonicalization_version != versions.canonicalization_version
        || e.digest_version != versions.digest_version
        || e.section_state.coverage() != SectionCoverage::Partial
        || e.section_state.quality() != SectionQuality::Unknown
        || e.section_state.freshness() != SectionFreshness::Fresh
        || e.section_state.availability() != SectionAvailability::Available
        || e.source_consistency != SourceConsistency::ObservedCountFillOnly
        || !e.stable_identity
    {
        return Err(rejected());
    }
    Ok(())
}

fn token(value: &str) -> bool {
    !value.is_empty()
        && value.len() <= 128
        && value
            .bytes()
            .all(|b| b.is_ascii_alphanumeric() || matches!(b, b'_' | b':' | b'-'))
}

fn field<'a>(fields: &mut std::str::Lines<'a>, prefix: &str) -> Result<&'a str, ProductionError> {
    fields
        .next()
        .and_then(|line| line.strip_prefix(prefix))
        .filter(|v| token(v))
        .ok_or_else(rejected)
}

fn validate_batch(
    v: &observation_domain::ImmutableBatchEnvelope,
    faction: &str,
) -> Result<(), ProductionError> {
    if v.records.is_empty() || v.optional_detail.is_some() || v.section_ordinal == 0 {
        return Err(rejected());
    }
    let mut prior_ordinal: Option<usize> = None;
    for record in &v.records {
        let ordinal = validate_record(record, faction, v.section_revision.get())?;
        if prior_ordinal.is_some_and(|prior| prior.checked_add(1) != Some(ordinal)) {
            return Err(rejected());
        }
        prior_ordinal = Some(ordinal);
    }
    Ok(())
}

fn validate_record(
    record: &observation_domain::EnvelopeRecord,
    faction: &str,
    revision: u64,
) -> Result<usize, ProductionError> {
    let mut fields = record.content.lines();
    if fields.next() != Some("profile=ship_core") {
        return Err(rejected());
    }
    let id = field(&mut fields, "identity=")?;
    let owner = field(&mut fields, "owner=")?;
    let kind = field(&mut fields, "type=")?;
    let class = field(&mut fields, "class=")?;
    let location = field(&mut fields, "location=")?;
    let consistency = ShipRecordConsistency::from_fields(
        field(&mut fields, "consistency=")?,
        field(&mut fields, "consistency_reason=")?,
    )
    .map_err(|_| rejected())?;
    if fields.next().is_some()
        || id.parse::<u64>().is_err()
        || ShipIdentity::new(id).is_err()
        || ShipOwner::new(owner).is_err()
        || ShipType::new(kind).is_err()
        || ShipClass::new(class).is_err()
        || ShipLocation::new(location).is_err()
        || record.entity_id.as_str() != format!("x4:ship:{id}")
        || record.observation_version.get() != revision
        || (owner != faction
            && consistency != ShipRecordConsistency::PossiblyStale(ShipStaleReason::OwnerChanged))
    {
        return Err(rejected());
    }
    let ordinal = record
        .record_id
        .as_str()
        .strip_prefix(&format!("carrier-b:{revision}:"))
        .filter(|value| value.len() == 20 && value.bytes().all(|byte| byte.is_ascii_digit()))
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .ok_or_else(rejected)?;
    Ok(ordinal)
}
