use super::Result;
use observation_domain::{EnvelopeRecord, SectionKey, SectionRevisionId};
use observation_persistence::{ObservationRepository, SqliteObservationRepository};
use std::collections::BTreeMap;

const EXPECTED: [(&str, &str); 11] = [
    (
        "argon",
        "disposition=included\nreason=independent_first_party\norigin=vanilla\nsource_evidence=base\nmind_candidate=true",
    ),
    (
        "scaleplate",
        "disposition=included\nreason=independent_first_party\norigin=vanilla\nsource_evidence=base\nmind_candidate=false",
    ),
    (
        "xenon",
        "disposition=included\nreason=independent_first_party\norigin=vanilla\nsource_evidence=base\nmind_candidate=false",
    ),
    (
        "khaak",
        "disposition=included\nreason=independent_first_party\norigin=vanilla\nsource_evidence=base\nmind_candidate=false",
    ),
    (
        "teladi",
        "disposition=included\nreason=independent_first_party\norigin=vanilla\nsource_evidence=base\nmind_candidate=false",
    ),
    (
        "player",
        "disposition=excluded\nreason=outside_mandatory_coverage\norigin=player\nsource_evidence=base\nmind_candidate=false",
    ),
    (
        "custom_mod",
        "disposition=unknown\nreason=origin_unresolved\norigin=\nsource_evidence=\nmind_candidate=false",
    ),
    (
        "civilian",
        "disposition=excluded\nreason=not_independent\norigin=service\nsource_evidence=base:hidden\nmind_candidate=false",
    ),
    (
        "criminal",
        "disposition=excluded\nreason=not_independent\norigin=service\nsource_evidence=base:hidden\nmind_candidate=false",
    ),
    (
        "outlaw",
        "disposition=excluded\nreason=not_independent\norigin=service\nsource_evidence=base:hidden\nmind_candidate=false",
    ),
    (
        "ownerless",
        "disposition=excluded\nreason=not_independent\norigin=service\nsource_evidence=base:hidden\nmind_candidate=false",
    ),
];

pub fn verify(
    repository: &SqliteObservationRepository,
    completions: &BTreeMap<String, Vec<u64>>,
) -> Result<()> {
    let revisions = completions
        .get("faction_census")
        .ok_or("missing durable census")?;
    if revisions.len() != 2 || revisions[0] == 0 || revisions[1] <= revisions[0] {
        return Err(format!("unexpected census revisions:{revisions:?}").into());
    }
    let key = SectionKey::new("faction_census").ok_or("invalid census key")?;
    let current = repository
        .current(&key)
        .map_err(|e| format!("census current:{e:?}"))?
        .ok_or("missing current census")?;
    if current.receipt().revision.get() != revisions[1] {
        return Err("wrong current census revision".into());
    }
    verify_records(&current.revision().records, revisions[1])?;
    for revision in revisions {
        let stored = repository
            .stored_revision(
                &key,
                SectionRevisionId::new(*revision).ok_or("invalid census revision")?,
            )
            .map_err(|e| format!("census history:{e:?}"))?
            .ok_or("missing retained census revision")?;
        if stored.1.revision.get() != *revision {
            return Err("census receipt revision mismatch".into());
        }
        verify_records(&stored.0.records, *revision)?;
    }
    Ok(())
}

fn verify_records(records: &[EnvelopeRecord], revision: u64) -> Result<()> {
    if records.len() != EXPECTED.len() {
        return Err(format!("census record count:{}", records.len()).into());
    }
    for (index, record) in records.iter().enumerate() {
        if record.record_id.as_str() != format!("carrier-b:{revision}:{:020}", index + 1) {
            return Err(format!("census record order:{revision}:{}", index + 1).into());
        }
    }
    for (id, claim) in EXPECTED {
        let entity = format!("x4:faction:{id}");
        let record = records
            .iter()
            .find(|record| record.entity_id.as_str() == entity)
            .ok_or_else(|| format!("missing census identity:{id}"))?;
        let content = format!("discovery_revision={revision}\n{claim}");
        if record.content != content {
            return Err(format!("census claim mismatch:{id}:{revision}").into());
        }
    }
    Ok(())
}
