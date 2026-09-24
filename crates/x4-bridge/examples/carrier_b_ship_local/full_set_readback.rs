use super::{Result, publication_limits};
use observation_domain::{CompletionCoverage, SectionKey, SectionRevisionId};
use observation_persistence::{ObservationRepository, SqliteObservationRepository};
use std::collections::BTreeMap;
use std::path::Path;
#[path = "full_set_census_readback.rs"]
mod census;

pub fn verify(database: &Path, completions: &BTreeMap<String, Vec<u64>>) -> Result<()> {
    let repository = SqliteObservationRepository::open(database, publication_limits()?)
        .map_err(|e| format!("full-set reopen:{e:?}"))?;
    census::verify(&repository, completions)?;
    let expected = [
        ("argon", 2usize, 3usize),
        ("scaleplate", 1, 2),
        ("xenon", 0, 3),
        ("khaak", 1, 3),
    ];
    for (faction, records, core_count) in expected {
        verify_key(
            &repository,
            completions,
            &format!("ship_core:{faction}"),
            records,
            core_count,
        )?;
        if records == 0 {
            verify_no_details(completions, faction)?;
            continue;
        }
        let minimum = detail_minimum(faction);
        for family in ["cargo", "crew", "loadout"] {
            verify_key(
                &repository,
                completions,
                &format!("ship_{family}:{faction}:g0"),
                records,
                minimum,
            )?;
        }
    }
    for forbidden in [
        "player",
        "custom_mod",
        "civilian",
        "criminal",
        "outlaw",
        "ownerless",
    ] {
        if completions.keys().any(|key| key.contains(forbidden)) {
            return Err(format!("disposed faction scheduled:{forbidden}").into());
        }
    }
    Ok(())
}

fn verify_no_details(completions: &BTreeMap<String, Vec<u64>>, faction: &str) -> Result<()> {
    for family in ["cargo", "crew", "loadout"] {
        if completions.contains_key(&format!("ship_{family}:{faction}:g0")) {
            return Err("zero-member faction published detail".into());
        }
    }
    Ok(())
}

fn detail_minimum(faction: &str) -> usize {
    usize::from(faction != "scaleplate") + 2
}

fn verify_key(
    repository: &SqliteObservationRepository,
    completions: &BTreeMap<String, Vec<u64>>,
    key: &str,
    records: usize,
    minimum: usize,
) -> Result<()> {
    let revisions = completions
        .get(key)
        .ok_or_else(|| format!("missing key:{key}"))?;
    if revisions.len() < minimum || revisions.len() < 2 {
        return Err(format!("insufficient revisions:{key}:{revisions:?}").into());
    }
    let section = SectionKey::new(key).ok_or("invalid readback key")?;
    let current = repository
        .current(&section)
        .map_err(|e| format!("current:{key}:{e:?}"))?
        .ok_or_else(|| format!("missing current:{key}"))?;
    if current.receipt().revision.get() != *revisions.last().ok_or("missing revision")?
        || current.revision().records.len() != records
        || current.revision().coverage != CompletionCoverage::Partial
    {
        return Err(format!("current mismatch:{key}").into());
    }
    for revision in revisions.iter().rev().take(2) {
        let stored = repository
            .stored_revision(
                &section,
                SectionRevisionId::new(*revision).ok_or("invalid revision")?,
            )
            .map_err(|e| format!("history:{key}:{e:?}"))?
            .ok_or_else(|| format!("missing retained revision:{key}:{revision}"))?;
        if stored.0.records.len() != records || stored.1.revision.get() != *revision {
            return Err(format!("history mismatch:{key}:{revision}").into());
        }
    }
    Ok(())
}
