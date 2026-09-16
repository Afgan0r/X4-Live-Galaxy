use super::{Result, publication_limits};
use observation_domain::{
    CrewObservation, FieldOutcome, LoadoutObservation, SectionKey, SectionRevisionId,
};
use observation_persistence::{ObservationRepository, SqliteObservationRepository};
use std::path::Path;

pub fn verify(database: &Path) -> Result<()> {
    let repository = SqliteObservationRepository::open(database, publication_limits()?)
        .map_err(|e| format!("independent crew reopen:{e:?}"))?;
    for group in 0..2 {
        for (family, first, last) in [
            ("ship_crew", 6 + group, 8 + group),
            ("ship_loadout", 10 + group, 12 + group),
        ] {
            verify_family(&repository, group, family, first, last)?;
        }
    }
    Ok(())
}
fn verify_family(
    repository: &SqliteObservationRepository,
    group: u64,
    family: &str,
    first: u64,
    last: u64,
) -> Result<()> {
    let key = SectionKey::new(format!("{family}:g{group}")).ok_or("key")?;
    let current = repository.current(&key).map_err(|e| format!("detail current:{e:?}"))?
                .ok_or("assertion failed: actual_chain_crew_and_installed_loadout_have_independent_durable_readback: missing detail revision")?;
    if current.receipt().revision.get() != last {
        return Err("detail current revision".into());
    }
    let verify_record = if family == "ship_crew" {
        verify_crew
    } else {
        verify_loadout
    };
    for number in [first, last] {
        let (revision, receipt) = repository
            .stored_revision(&key, SectionRevisionId::new(number).ok_or("revision")?)
            .map_err(|e| format!("detail history:{e:?}"))?
            .ok_or("detail history missing")?;
        if revision.records.len() != 4
            || receipt.revision.get() != number
            || revision
                .dependencies
                .get(&SectionKey::new("ship_core").ok_or("key")?)
                .map(|v| v.get())
                != Some(1)
        {
            return Err("detail exact durable dependency".into());
        }
        for record in &revision.records {
            verify_record(&record.content, number)?;
        }
    }
    Ok(())
}
fn verify_crew(content: &str, revision: u64) -> Result<()> {
    let value =
        CrewObservation::from_content(content, 16).map_err(|e| format!("crew decode:{e:?}"))?;
    let FieldOutcome::Value(roles) = value.roles else {
        return Err("crew roles outcome".into());
    };
    if value.capacity != FieldOutcome::Value(u32::try_from(12 + revision)?)
        || roles.len() != 2
        || roles[0].id != "passenger"
        || roles[0].canhire
        || roles[0].amount_people != 2
        || roles[1].id != "service"
        || !roles[1].canhire
        || roles[1].tiers[0].skill_lower_threshold != -25
    {
        return Err("crew all roles/raw signed tiers".into());
    }
    Ok(())
}
fn verify_loadout(content: &str, revision: u64) -> Result<()> {
    let value = LoadoutObservation::from_content(content, 16)
        .map_err(|e| format!("loadout decode:{e:?}"))?;
    let (
        FieldOutcome::Value(physical),
        FieldOutcome::Value(virtual_slots),
        FieldOutcome::Value(software),
        FieldOutcome::Value(missiles),
        FieldOutcome::Value(units),
    ) = (
        value.physical,
        value.virtual_slots,
        value.software,
        value.missiles,
        value.units,
    )
    else {
        return Err("loadout required family outcomes".into());
    };
    if physical.len() != 4
        || physical.iter().any(|r| {
            r.component != "0"
                || r.path != ".."
                || !r.group.is_empty()
                || r.macro_name != format!("{}_installed_macro", r.kind)
        })
        || virtual_slots[0].macro_name != "thruster_current_macro"
        || software[0].current != "software_current"
        || i64::from(missiles[0].amount_raw) != -i64::try_from(revision)?
        || units.len() != 2
        || units[1].category != "unfiltered_raw"
    {
        return Err("installed macro-only/group/software/signed missiles/all units".into());
    }
    Ok(())
}
