use super::{ProductionError, ProductionObservationSession};
use observation_domain::{
    FactionObservationDisposition, FactionOriginEvidence, ShipSectionIdentity, ShipSectionKind,
    SourceScopeId,
};
use observation_persistence::ObservationRepository;

impl<R: ObservationRepository> ProductionObservationSession<R> {
    pub fn select_ship_core(&mut self, faction: &str) -> Result<(), ProductionError> {
        self.select_ship_factions([faction])
    }

    pub fn select_ship_factions<'a>(
        &mut self,
        factions: impl IntoIterator<Item = &'a str>,
    ) -> Result<(), ProductionError> {
        let mut scopes = Vec::new();
        for faction in factions {
            self.validate_included_faction(faction)?;
            push_unique_scope(&mut scopes, faction)?;
        }
        let first = scopes
            .first()
            .cloned()
            .ok_or(ProductionError::InvalidLimits)?;
        self.ship_scopes = scopes;
        self.ship_scope_index = 0;
        self.ship_scope = Some(first);
        self.ship_timing.clear();
        self.last_received = None;
        Ok(())
    }

    pub fn accept_faction_census<I, S>(
        &mut self,
        discovery_revision: u64,
        discovered: I,
        inventory: &[FactionOriginEvidence],
    ) -> Result<(), ProductionError>
    where
        I: IntoIterator<Item = S>,
        S: Into<String>,
    {
        let prior = self.faction_roster.as_ref().map_or(
            0,
            observation_domain::FactionObservationRoster::discovery_revision,
        );
        if discovery_revision <= prior {
            return Err(ProductionError::InvalidFactionCensus);
        }
        let roster = observation_domain::classify_observation_factions(
            discovery_revision,
            discovered,
            inventory,
        )
        .map_err(|_| ProductionError::InvalidFactionCensus)?;
        let included = roster.included_ids().map(str::to_owned).collect::<Vec<_>>();
        self.faction_census_mode = true;
        self.faction_roster = Some(roster);
        self.ship_scope = None;
        self.ship_scopes.clear();
        self.ship_scope_index = 0;
        self.ship_timing.clear();
        self.last_received = None;
        if !included.is_empty() {
            self.select_ship_factions(included.iter().map(String::as_str))?;
        }
        Ok(())
    }

    #[must_use]
    pub fn faction_census_blocks_closure(&self) -> bool {
        self.faction_roster
            .as_ref()
            .is_none_or(observation_domain::FactionObservationRoster::has_unknown_blocker)
    }

    pub fn included_faction_ids(&self) -> Vec<String> {
        self.faction_roster
            .as_ref()
            .map_or_else(Vec::new, |roster| {
                roster.included_ids().map(str::to_owned).collect()
            })
    }

    pub fn refresh_faction_census(&mut self) {
        self.faction_roster = None;
        self.ship_scope = None;
        self.ship_scopes.clear();
        self.ship_scope_index = 0;
        self.ship_timing.clear();
        self.last_received = None;
    }

    fn validate_included_faction(&self, faction: &str) -> Result<(), ProductionError> {
        self.faction_roster
            .as_ref()
            .and_then(|roster| roster.entry(faction))
            .filter(|entry| entry.disposition() == FactionObservationDisposition::Included)
            .ok_or(ProductionError::InvalidFactionCensus)
            .map(|_| ())
    }

    pub fn collection_key(&self) -> String {
        if !self.faction_census_mode {
            return "carrier_b_realtime_sample".to_owned();
        }
        if self.faction_roster.is_none() {
            return "faction_census".to_owned();
        }
        self.ship_scope.as_ref().map_or_else(
            || "carrier_b_realtime_sample".to_owned(),
            |_| {
                self.scoped_key("ship_core")
                    .unwrap_or_else(|_| "ship_core".to_owned())
            },
        )
    }

    pub(super) fn scoped_key(&self, legacy: &str) -> Result<String, ProductionError> {
        let section = ShipSectionIdentity::parse(legacy).ok_or(ProductionError::InvalidLimits)?;
        if !self.faction_census_mode {
            return Ok(section.key());
        }
        let faction = self.ship_faction().ok_or(ProductionError::InvalidLimits)?;
        let scoped = match section.kind() {
            ShipSectionKind::Core => ShipSectionIdentity::core(Some(faction)),
            kind => ShipSectionIdentity::detail(
                kind,
                Some(faction),
                section.group().ok_or(ProductionError::InvalidLimits)?,
            ),
        };
        scoped
            .map(|value| value.key())
            .ok_or(ProductionError::InvalidLimits)
    }

    fn ship_faction(&self) -> Option<&str> {
        self.ship_scope
            .as_ref()?
            .as_str()
            .strip_prefix("x4:faction:")?
            .strip_suffix(":ships")
    }

    pub(super) fn rotate_ship_scope(&mut self) {
        if self.ship_scopes.len() > 1 {
            self.ship_scope_index = (self.ship_scope_index + 1) % self.ship_scopes.len();
            self.ship_scope = self.ship_scopes.get(self.ship_scope_index).cloned();
            self.ship_timing.clear();
            self.last_received = None;
        }
    }

    pub fn skip_stale_ship_scope(&mut self) -> Result<String, ProductionError> {
        if self.ship_scopes.len() <= 1 {
            return Err(ProductionError::InvalidLimits);
        }
        self.rotate_ship_scope();
        Ok(self.collection_key())
    }
}

fn push_unique_scope(
    scopes: &mut Vec<SourceScopeId>,
    faction: &str,
) -> Result<(), ProductionError> {
    let scope = crate::receiver_ship::scope(faction)?;
    if scopes.contains(&scope) {
        return Err(ProductionError::InvalidLimits);
    }
    scopes.push(scope);
    Ok(())
}
