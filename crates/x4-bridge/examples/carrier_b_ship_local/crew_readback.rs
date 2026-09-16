use super::{Result, publication_limits};
use std::path::Path;
use observation_domain::SectionKey;
use observation_persistence::{ObservationRepository, SqliteObservationRepository};

pub fn verify(database: &Path) -> Result<()> {
    let repository = SqliteObservationRepository::open(database, publication_limits()?).map_err(|e| format!("independent crew reopen:{e:?}"))?;
    for name in ["ship_crew:g0", "ship_loadout:g0"] {
        let key = SectionKey::new(name).ok_or("key")?;
        if repository.current(&key).map_err(|e| format!("detail current:{e:?}"))?.is_none() {
            return Err("assertion failed: actual_chain_crew_and_installed_loadout_have_independent_durable_readback: missing detail revision".into());
        }
    }
    Ok(())
}
