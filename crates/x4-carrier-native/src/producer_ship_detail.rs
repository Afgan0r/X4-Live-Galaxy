use crate::producer_types::{PreparedRecord, ProducerProfile};
use crate::{Producer, ProducerError};

impl Producer {
    pub fn push_ship_loadout(
        &mut self,
        scope: &str,
        record: &observation_domain::LoadoutObservation,
    ) -> Result<(), ProducerError> {
        if self.profile != ProducerProfile::ShipLoadout
            || self
                .evidence
                .as_ref()
                .is_none_or(|e| e.source_scope != scope)
        {
            return Err(ProducerError::InvalidInput);
        }
        record
            .validate(self.limits.max_records)
            .map_err(|_| ProducerError::InvalidInput)?;
        let content = record.canonical_content();
        if content.len() > self.limits.max_raw_bytes {
            return Err(ProducerError::DataLimit);
        }
        self.push(PreparedRecord {
            entity_id: format!("x4:ship:{}", record.dependency.identity.as_str()),
            content,
        })
    }
    pub fn push_ship_crew(
        &mut self,
        scope: &str,
        record: &observation_domain::CrewObservation,
    ) -> Result<(), ProducerError> {
        if self.profile != ProducerProfile::ShipCrew
            || self
                .evidence
                .as_ref()
                .is_none_or(|e| e.source_scope != scope)
        {
            return Err(ProducerError::InvalidInput);
        }
        record
            .validate(self.limits.max_records)
            .map_err(|_| ProducerError::InvalidInput)?;
        let content = record.canonical_content();
        if content.len() > self.limits.max_raw_bytes {
            return Err(ProducerError::DataLimit);
        }
        self.push(PreparedRecord {
            entity_id: format!("x4:ship:{}", record.dependency.identity.as_str()),
            content,
        })
    }
    pub fn push_ship_cargo(
        &mut self,
        scope: &str,
        record: &observation_domain::CargoObservation,
    ) -> Result<(), ProducerError> {
        if self.profile != ProducerProfile::ShipCargo
            || self
                .evidence
                .as_ref()
                .is_none_or(|e| e.source_scope != scope)
        {
            return Err(ProducerError::InvalidInput);
        }
        record
            .validate(self.limits.max_records)
            .map_err(|_| ProducerError::InvalidInput)?;
        let content = record.canonical_content();
        if content.len() > self.limits.max_raw_bytes {
            return Err(ProducerError::DataLimit);
        }
        self.push(PreparedRecord {
            entity_id: format!("x4:ship:{}", record.dependency.identity.as_str()),
            content,
        })
    }
}
