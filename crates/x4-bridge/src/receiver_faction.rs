use super::{ProductionError, ProductionObservationSession};
use observation_domain::{
    EnvelopeRecord, FactionObservationDisposition, FactionObservationRoster, FactionOrigin,
    FactionOriginEvidence, SectionKey,
};
use observation_persistence::ObservationRepository;

impl<R: ObservationRepository> ProductionObservationSession<R> {
    pub(super) fn submit_validated_census(
        &mut self,
        input: observation_application::LifecycleInput,
    ) -> Result<observation_application::LifecycleResult, observation_application::LifecycleError>
    {
        let inventory = self.faction_inventory.clone();
        self.lifecycle
            .submit_validated(input, &|revision| validates_revision(revision, &inventory))
    }

    pub fn configure_faction_inventory(
        &mut self,
        inventory: Vec<FactionOriginEvidence>,
    ) -> Result<(), ProductionError> {
        let _validated =
            observation_domain::classify_observation_factions(1, Vec::<String>::new(), &inventory)
                .map_err(|_| ProductionError::InvalidFactionCensus)?;
        self.faction_inventory = inventory;
        self.faction_roster = None;
        self.faction_census_mode = true;
        Ok(())
    }

    pub(super) fn accept_committed_faction_census(&mut self) -> Result<(), ProductionError> {
        let key = SectionKey::new("faction_census").ok_or(ProductionError::InvalidFactionCensus)?;
        let accepted = self
            .lifecycle
            .current_revision(&key)
            .map_err(|_| ProductionError::Storage)?
            .ok_or(ProductionError::InvalidFactionCensus)?;
        let claims = &accepted.revision().records;
        let revision = accepted.receipt().revision.get();
        let roster = validate_census(revision, claims, &self.faction_inventory)?;
        self.accept_faction_census(
            revision,
            roster.entries().iter().map(|entry| entry.id().to_owned()),
            &self.faction_inventory.clone(),
        )
    }
}

pub(super) fn validate_census(
    revision: u64,
    records: &[EnvelopeRecord],
    inventory: &[FactionOriginEvidence],
) -> Result<FactionObservationRoster, ProductionError> {
    let claims = records
        .iter()
        .map(|record| parse_claim(record.entity_id.as_str(), &record.content))
        .collect::<Result<Vec<_>, _>>()?;
    if claims.is_empty() || claims.iter().any(|claim| claim.revision != revision) {
        return Err(ProductionError::InvalidFactionCensus);
    }
    let roster = observation_domain::classify_observation_factions(
        revision,
        claims.iter().map(|claim| claim.id.clone()),
        inventory,
    )
    .map_err(|_| ProductionError::InvalidFactionCensus)?;
    claims
        .iter()
        .all(|claim| claim.matches(roster.entry(&claim.id)))
        .then_some(roster)
        .ok_or(ProductionError::InvalidFactionCensus)
}

pub(super) fn validates_revision(
    revision: &observation_ingest::ValidatedSectionRevision,
    inventory: &[FactionOriginEvidence],
) -> bool {
    validate_census(
        revision.section_revision().get(),
        revision.records(),
        inventory,
    )
    .is_ok()
}

struct Claim {
    id: String,
    revision: u64,
    disposition: String,
    reason: String,
    origin: String,
    source: String,
    mind: bool,
}

impl Claim {
    fn matches(&self, entry: Option<&observation_domain::FactionCensusEntry>) -> bool {
        let Some(entry) = entry else { return false };
        self.disposition == disposition(entry.disposition())
            && self.reason.replace('_', "-") == entry.reason()
            && self.origin == entry.origin().map_or("", origin)
            && self.source == entry.source_evidence().map_or("", |value| value.as_str())
            && self.mind == entry.mind_candidate()
    }
}

fn parse_claim(entity: &str, content: &str) -> Result<Claim, ProductionError> {
    let id = entity
        .strip_prefix("x4:faction:")
        .ok_or(ProductionError::InvalidFactionCensus)?
        .to_owned();
    let fields = content
        .lines()
        .filter_map(|line| line.split_once('='))
        .collect::<std::collections::BTreeMap<_, _>>();
    (fields.len() == 6)
        .then_some(Claim {
            id,
            revision: fields
                .get("discovery_revision")
                .and_then(|v| v.parse().ok())
                .ok_or(ProductionError::InvalidFactionCensus)?,
            disposition: required(&fields, "disposition")?.to_owned(),
            reason: required(&fields, "reason")?.to_owned(),
            origin: required(&fields, "origin")?.to_owned(),
            source: required(&fields, "source_evidence")?.to_owned(),
            mind: required(&fields, "mind_candidate")?
                .parse()
                .map_err(|_| ProductionError::InvalidFactionCensus)?,
        })
        .ok_or(ProductionError::InvalidFactionCensus)
}

fn required<'a>(
    fields: &std::collections::BTreeMap<&'a str, &'a str>,
    key: &str,
) -> Result<&'a str, ProductionError> {
    fields
        .get(key)
        .copied()
        .ok_or(ProductionError::InvalidFactionCensus)
}

const fn disposition(value: FactionObservationDisposition) -> &'static str {
    match value {
        FactionObservationDisposition::Included => "included",
        FactionObservationDisposition::Excluded => "excluded",
        FactionObservationDisposition::Unknown => "unknown",
    }
}

const fn origin(value: FactionOrigin) -> &'static str {
    match value {
        FactionOrigin::Vanilla => "vanilla",
        FactionOrigin::Dlc => "dlc",
        FactionOrigin::Player => "player",
        FactionOrigin::Service => "service",
        FactionOrigin::Modded => "modded",
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use observation_domain::{EntityId, ObservationVersion, RecordId};

    fn record(revision: u64) -> EnvelopeRecord {
        EnvelopeRecord {
            record_id: RecordId::new("record:argon").expect("record"),
            entity_id: EntityId::new("x4:faction:argon").expect("entity"),
            observation_version: ObservationVersion::new(1).expect("version"),
            content: format!(
                "discovery_revision={revision}\ndisposition=included\nreason=independent-first-party\norigin=vanilla\nsource_evidence=base\nmind_candidate=true"
            ),
        }
    }

    fn inventory() -> Vec<FactionOriginEvidence> {
        vec![
            FactionOriginEvidence::new("argon", FactionOrigin::Vanilla, Some(true), true, "base")
                .expect("inventory"),
        ]
    }

    #[test]
    fn census_claim_revision_must_equal_section_revision() {
        assert!(validate_census(2, &[record(1)], &inventory()).is_err());
        assert!(validate_census(2, &[record(2)], &inventory()).is_ok());
    }
}
