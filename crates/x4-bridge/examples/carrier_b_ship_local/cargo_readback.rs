use super::{Result, publication_limits};
use observation_domain::{CargoObservation, FieldOutcome, SectionKey, SectionRevisionId};
use observation_persistence::{ObservationRepository, RevisionRecord, SqliteObservationRepository};
use std::path::Path;

pub fn verify(database: &Path) -> Result<()> {
    let repository = SqliteObservationRepository::open(database, publication_limits()?)
        .map_err(|e| format!("independent cargo reopen:{e:?}"))?;
    for group in 0..2 {
        verify_group(&repository, group)?;
    }
    Ok(())
}
fn verify_group(repository: &SqliteObservationRepository, group: u64) -> Result<()> {
    let key = SectionKey::new(format!("ship_cargo:g{group}")).ok_or("key")?;
    let current = repository.current(&key).map_err(|e| format!("current:{e:?}"))?.ok_or("assertion failed: actual_chain_cargo_has_independent_durable_readback: missing cargo revision")?;
    if current.receipt().revision.get() != 4 + group {
        return Err("cargo current revision".into());
    }
    for number in [2 + group, 4 + group] {
        let (revision, receipt) = repository
            .stored_revision(&key, SectionRevisionId::new(number).ok_or("revision")?)
            .map_err(|e| format!("history:{e:?}"))?
            .ok_or("history missing")?;
        if receipt.revision.get() != number {
            return Err("cargo receipt revision".into());
        }
        verify_revision(&revision, group, number)?;
    }
    Ok(())
}
fn verify_revision(revision: &RevisionRecord, group: u64, number: u64) -> Result<()> {
    if revision.records.len() != 4
        || revision
            .dependencies
            .get(&SectionKey::new("ship_core").ok_or("key")?)
            .map(|v| v.get())
            != Some(1)
    {
        return Err("cargo durable dependency".into());
    }
    for (index, record) in revision.records.iter().enumerate() {
        let cargo = CargoObservation::from_content(&record.content, 16)
            .map_err(|e| format!("decode:{e:?}"))?;
        verify_record(&cargo, group, number, index)?;
    }
    Ok(())
}
fn verify_record(cargo: &CargoObservation, group: u64, number: u64, index: usize) -> Result<()> {
    let identity = format!("900719925474099{}", group * 4 + index as u64 + 2);
    if cargo.dependency.identity.as_str() != identity
        || cargo.dependency.capture.start_millis() != 20 + number
        || cargo.dependency.capture.end_millis() != 30 + number
    {
        return Err("cargo identity/capture history".into());
    }
    let FieldOutcome::Value(wares) = &cargo.wares else {
        return Err("cargo ware outcome".into());
    };
    let FieldOutcome::Value(storage) = &cargo.storage else {
        return Err("cargo storage outcome".into());
    };
    if wares.len() != 2
        || wares[0].ware != "energycells"
        || wares[0].amount_items != 7
        || wares[1].amount_items != 11
        || storage.len() != 1
        || storage[0].capacity_cubic_metres != 1200
        || storage[0].occupied_cubic_metres != 110
    {
        return Err("cargo raw units".into());
    }
    Ok(())
}
