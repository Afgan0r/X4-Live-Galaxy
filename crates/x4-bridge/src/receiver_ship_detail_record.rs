use crate::ProductionError;
use observation_domain::{
    CargoObservation, CrewObservation, ImmutableBatchEnvelope, LoadoutObservation,
    ShipDetailDependency,
};
const fn rejected() -> ProductionError {
    ProductionError::Lifecycle(observation_application::LifecycleError::AuthorityRejected)
}
pub fn validate_batch(
    batch: &ImmutableBatchEnvelope,
    faction: &str,
) -> Result<(), ProductionError> {
    if batch.records.is_empty() || batch.optional_detail.is_some() || batch.section_ordinal == 0 {
        return Err(rejected());
    }
    let profile = batch
        .section_key
        .as_str()
        .split_once(":g")
        .map(|v| v.0)
        .ok_or_else(rejected)?;
    let mut prior_ordinal = None;
    for record in &batch.records {
        if !record.content.starts_with(&format!("profile={profile}\n")) {
            return Err(rejected());
        }
        let dependency = record_dependency(&record.content)?;
        let ordinal = record_ordinal(record, batch.section_revision.get())?;
        if dependency.owner.as_str() != faction
            || record.entity_id.as_str() != format!("x4:ship:{}", dependency.identity.as_str())
            || record.observation_version.get() != batch.section_revision.get()
            || prior_ordinal.is_some_and(|prior: usize| prior.checked_add(1) != Some(ordinal))
        {
            return Err(rejected());
        }
        prior_ordinal = Some(ordinal);
    }
    Ok(())
}

fn record_ordinal(
    record: &observation_domain::EnvelopeRecord,
    revision: u64,
) -> Result<usize, ProductionError> {
    record
        .record_id
        .as_str()
        .strip_prefix(&format!("carrier-b:{revision}:"))
        .filter(|value| value.len() == 20 && value.bytes().all(|byte| byte.is_ascii_digit()))
        .and_then(|value| value.parse::<usize>().ok())
        .filter(|value| *value > 0)
        .ok_or_else(rejected)
}
pub fn record_dependency(content: &str) -> Result<ShipDetailDependency, ProductionError> {
    // Each nested row consumes at least one wire byte. The enclosing admitted
    // frame/candidate bounds content; this is not a world population quota.
    record_dependency_with_limit(content, content.len())
}
pub fn record_dependency_with_limit(
    content: &str,
    limit: usize,
) -> Result<ShipDetailDependency, ProductionError> {
    if content.starts_with("profile=ship_cargo\n") {
        Ok(CargoObservation::from_content(content, limit)
            .map_err(|_| rejected())?
            .dependency)
    } else if content.starts_with("profile=ship_crew\n") {
        Ok(CrewObservation::from_content(content, limit)
            .map_err(|_| rejected())?
            .dependency)
    } else if content.starts_with("profile=ship_loadout\n") {
        Ok(LoadoutObservation::from_content(content, limit)
            .map_err(|_| rejected())?
            .dependency)
    } else {
        Err(rejected())
    }
}
