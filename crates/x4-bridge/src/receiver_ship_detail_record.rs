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
    if batch.records.len() != 1 || batch.optional_detail.is_some() || batch.section_ordinal == 0 {
        return Err(rejected());
    }
    let record = batch.records.first().ok_or_else(rejected)?;
    let profile = batch
        .section_key
        .as_str()
        .split_once(":g")
        .map(|v| v.0)
        .ok_or_else(rejected)?;
    if !record.content.starts_with(&format!("profile={profile}\n")) {
        return Err(rejected());
    }
    let dependency = record_dependency(&record.content)?;
    if dependency.owner.as_str() != faction
        || record.entity_id.as_str() != format!("x4:ship:{}", dependency.identity.as_str())
        || record.record_id.as_str()
            != format!(
                "carrier-b:{}:{:020}",
                batch.section_revision.get(),
                batch.section_ordinal
            )
        || record.observation_version.get() != batch.section_revision.get()
    {
        return Err(rejected());
    }
    Ok(())
}
pub fn record_dependency(content: &str) -> Result<ShipDetailDependency, ProductionError> {
    if content.starts_with("profile=ship_cargo\n") {
        Ok(CargoObservation::from_content(content, 64)
            .map_err(|_| rejected())?
            .dependency)
    } else if content.starts_with("profile=ship_crew\n") {
        Ok(CrewObservation::from_content(content, 64)
            .map_err(|_| rejected())?
            .dependency)
    } else if content.starts_with("profile=ship_loadout\n") {
        Ok(LoadoutObservation::from_content(content, 64)
            .map_err(|_| rejected())?
            .dependency)
    } else {
        Err(rejected())
    }
}
