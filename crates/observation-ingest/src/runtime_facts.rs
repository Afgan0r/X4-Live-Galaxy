use observation_domain::EntityId;
use serde::Deserialize;

use crate::model::AdmissionError;

pub const MAX_RUNTIME_FACTS_PER_CLASS: usize = 16;
pub const MAX_RUNTIME_FACT_STRING_BYTES: usize = 96;

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RuntimeFacts {
    #[serde(rename = "r")]
    pub source: String,
    #[serde(rename = "g", default)]
    pub x4_game_time: Option<u64>,
    #[serde(rename = "q")]
    pub quality: RuntimeFactQuality,
    #[serde(rename = "a")]
    pub availability: RuntimeFactAvailability,
    #[serde(rename = "s")]
    pub sectors: Vec<RuntimeSector>,
    #[serde(rename = "x")]
    pub assets: Vec<RuntimeAsset>,
    #[serde(rename = "c")]
    pub capacity: Vec<RuntimeCapacity>,
    #[serde(rename = "o")]
    pub ownership: Vec<RuntimeOwnership>,
    #[serde(skip)]
    pub receipt_unix_millis: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RuntimeSector {
    #[serde(rename = "i")]
    pub id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RuntimeAsset {
    #[serde(rename = "i")]
    pub id: String,
    #[serde(rename = "p")]
    pub sector_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RuntimeCapacity {
    #[serde(rename = "i")]
    pub id: String,
    #[serde(rename = "p")]
    pub asset_id: String,
    #[serde(rename = "v")]
    pub value: u64,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(deny_unknown_fields)]
pub struct RuntimeOwnership {
    #[serde(rename = "i")]
    pub id: String,
    #[serde(rename = "p")]
    pub asset_id: String,
    #[serde(rename = "n")]
    pub owner_id: String,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeFactQuality {
    Fresh,
    KnownEmpty,
    Unknown,
    Partial,
    Stale,
    Unsupported,
}

#[derive(Clone, Debug, Deserialize, Eq, PartialEq)]
#[serde(rename_all = "snake_case")]
pub enum RuntimeFactAvailability {
    Available,
    Unavailable,
}

impl RuntimeFacts {
    pub(crate) fn validate(&self, sector_id: &EntityId) -> Result<(), AdmissionError> {
        if !within_bounds(self) {
            return Err(AdmissionError::CollectionLimitExceeded);
        }
        let valid = self
            .sectors
            .iter()
            .any(|sector| sector.id == sector_id.as_str())
            && self.assets.iter().all(|asset| {
                self.sectors
                    .iter()
                    .any(|sector| sector.id == asset.sector_id)
            })
            && self.capacity.iter().all(|capacity| {
                self.assets
                    .iter()
                    .any(|asset| asset.id == capacity.asset_id)
            })
            && self.ownership.iter().all(|ownership| {
                self.assets
                    .iter()
                    .any(|asset| asset.id == ownership.asset_id)
                    && EntityId::new(ownership.owner_id.clone()).is_some()
            })
            && self.source == "x4_runtime"
            && matches!(self.availability, RuntimeFactAvailability::Available)
            && all_bounded(self)
            && all_identities_valid(self)
            && sorted_unique(&self.sectors, |item| &item.id)
            && sorted_unique(&self.assets, |item| &item.id)
            && sorted_unique(&self.capacity, |item| &item.id)
            && sorted_unique(&self.ownership, |item| &item.id);
        let valid_game_time = self
            .x4_game_time
            .is_none_or(|game_time| game_time <= 9_007_199_254_740_991);
        (valid && valid_game_time)
            .then_some(())
            .ok_or(AdmissionError::InvalidContent)
    }
}

fn within_bounds(facts: &RuntimeFacts) -> bool {
    [
        facts.sectors.len(),
        facts.assets.len(),
        facts.capacity.len(),
        facts.ownership.len(),
    ]
    .iter()
    .all(|count| (1..=MAX_RUNTIME_FACTS_PER_CLASS).contains(count))
}

fn all_bounded(facts: &RuntimeFacts) -> bool {
    facts.source.len() <= MAX_RUNTIME_FACT_STRING_BYTES
        && facts.sectors.iter().all(|item| bounded(&item.id))
        && facts
            .assets
            .iter()
            .all(|item| bounded(&item.id) && bounded(&item.sector_id))
        && facts
            .capacity
            .iter()
            .all(|item| bounded(&item.id) && bounded(&item.asset_id))
        && facts
            .ownership
            .iter()
            .all(|item| bounded(&item.id) && bounded(&item.asset_id) && bounded(&item.owner_id))
}

fn all_identities_valid(facts: &RuntimeFacts) -> bool {
    facts.sectors.iter().all(|item| valid_id(&item.id))
        && facts
            .assets
            .iter()
            .all(|item| valid_id(&item.id) && valid_id(&item.sector_id))
        && facts
            .capacity
            .iter()
            .all(|item| valid_id(&item.id) && valid_id(&item.asset_id))
        && facts
            .ownership
            .iter()
            .all(|item| valid_id(&item.id) && valid_id(&item.asset_id) && valid_id(&item.owner_id))
}

const fn bounded(value: &str) -> bool {
    value.len() <= MAX_RUNTIME_FACT_STRING_BYTES
}

fn valid_id(value: &str) -> bool {
    EntityId::new(value.to_owned()).is_some()
}

fn sorted_unique<T>(items: &[T], id: impl Fn(&T) -> &str) -> bool {
    items.windows(2).all(|pair| id(&pair[0]) < id(&pair[1]))
}
