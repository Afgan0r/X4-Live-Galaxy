use std::collections::{BTreeMap, BTreeSet};
use std::fmt;

use observation_domain::{FactionOrigin, FactionOriginEvidence};
use serde::de::{MapAccess, Visitor};
use serde::{Deserialize, Deserializer};

pub const MAX_DOCUMENT_BYTES: usize = 64 * 1_024;
const SCHEMA_VERSION: u8 = 1;
const GAME_BUILD: &str = "9.00-steam-23660954";
const MAX_ENTRIES: usize = 256;
const MAX_SOURCES: usize = 32;

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Inventory {
    schema_version: u8,
    game_build: String,
    #[serde(deserialize_with = "unique_sources")]
    sources: BTreeMap<String, String>,
    entries: Vec<Entry>,
}

#[derive(Deserialize)]
#[serde(deny_unknown_fields)]
struct Entry {
    id: String,
    origin: Origin,
    independent: Option<bool>,
    mind_candidate: bool,
    evidence: String,
    #[serde(rename = "classification")]
    _classification: Classification,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Origin {
    Vanilla,
    Dlc,
    Player,
    Service,
    Modded,
}

#[derive(Deserialize)]
#[serde(rename_all = "lowercase")]
enum Classification {
    Documented,
    Inferred,
}

fn unique_sources<'de, D>(deserializer: D) -> Result<BTreeMap<String, String>, D::Error>
where
    D: Deserializer<'de>,
{
    deserializer.deserialize_map(UniqueSources)
}

struct UniqueSources;
impl<'de> Visitor<'de> for UniqueSources {
    type Value = BTreeMap<String, String>;

    fn expecting(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
        formatter.write_str("a source map without duplicate keys")
    }

    fn visit_map<A>(self, mut map: A) -> Result<Self::Value, A::Error>
    where
        A: MapAccess<'de>,
    {
        let mut sources = BTreeMap::new();
        while let Some((key, value)) = map.next_entry::<String, String>()? {
            insert_unique(&mut sources, key, value)?;
        }
        Ok(sources)
    }
}

fn insert_unique<E: serde::de::Error>(
    sources: &mut BTreeMap<String, String>,
    key: String,
    value: String,
) -> Result<(), E> {
    if sources.insert(key, value).is_some() {
        return Err(E::custom("duplicate source key"));
    }
    Ok(())
}

pub fn decode(bytes: &[u8]) -> Option<Vec<FactionOriginEvidence>> {
    let inventory: Inventory = serde_json::from_slice(bytes).ok()?;
    inventory.validate()
}

impl Inventory {
    fn validate(self) -> Option<Vec<FactionOriginEvidence>> {
        if self.schema_version != SCHEMA_VERSION
            || self.game_build != GAME_BUILD
            || self.sources.is_empty()
            || self.sources.len() > MAX_SOURCES
            || self.entries.is_empty()
            || self.entries.len() > MAX_ENTRIES
            || self
                .sources
                .iter()
                .any(|(key, value)| !valid_text(key, 32) || !valid_text(value, 256))
        {
            return None;
        }
        let mut ids = BTreeSet::new();
        self.entries
            .into_iter()
            .map(|entry| entry.validate(&self.sources, &mut ids))
            .collect()
    }
}

impl Entry {
    fn validate(
        self,
        sources: &BTreeMap<String, String>,
        ids: &mut BTreeSet<String>,
    ) -> Option<FactionOriginEvidence> {
        if !valid_text(&self.id, 64)
            || !valid_text(&self.evidence, 128)
            || !ids.insert(self.id.clone())
            || !evidence_known(&self.evidence, sources)
        {
            return None;
        }
        let origin = match self.origin {
            Origin::Vanilla => FactionOrigin::Vanilla,
            Origin::Dlc => FactionOrigin::Dlc,
            Origin::Player => FactionOrigin::Player,
            Origin::Service => FactionOrigin::Service,
            Origin::Modded => FactionOrigin::Modded,
        };
        FactionOriginEvidence::new(
            self.id,
            origin,
            self.independent,
            self.mind_candidate,
            self.evidence,
        )
        .ok()
    }
}

fn valid_text(value: &str, max: usize) -> bool {
    !value.is_empty() && value.len() <= max && value.is_ascii() && !value.contains(['"', '\\'])
}

fn evidence_known(evidence: &str, sources: &BTreeMap<String, String>) -> bool {
    evidence.split('+').all(|part| {
        let base = part.split(':').next().unwrap_or(part);
        sources.contains_key(base)
            || base
                .strip_suffix("-patch")
                .is_some_and(|key| sources.contains_key(key))
    })
}
