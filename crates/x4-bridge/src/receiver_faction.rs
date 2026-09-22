use super::{ProductionError, ProductionObservationSession};
use observation_domain::{
    FactionObservationDisposition, FactionOrigin, FactionOriginEvidence, SectionKey,
};
use observation_persistence::ObservationRepository;

impl<R: ObservationRepository> ProductionObservationSession<R> {
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
        let claims = accepted
            .revision()
            .records
            .iter()
            .map(|record| parse_claim(record.entity_id.as_str(), &record.content))
            .collect::<Result<Vec<_>, _>>()?;
        let revision = claims
            .first()
            .map(|claim| claim.revision)
            .ok_or(ProductionError::InvalidFactionCensus)?;
        if claims.iter().any(|claim| claim.revision != revision) {
            return Err(ProductionError::InvalidFactionCensus);
        }
        let roster = observation_domain::classify_observation_factions(
            revision,
            claims.iter().map(|claim| claim.id.clone()),
            &self.faction_inventory,
        )
        .map_err(|_| ProductionError::InvalidFactionCensus)?;
        if claims
            .iter()
            .any(|claim| !claim.matches(roster.entry(&claim.id)))
        {
            return Err(ProductionError::InvalidFactionCensus);
        }
        self.accept_faction_census(
            revision,
            claims.into_iter().map(|claim| claim.id),
            &self.faction_inventory.clone(),
        )
    }
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
