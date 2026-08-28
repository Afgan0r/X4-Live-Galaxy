use crate::fact::{FactAvailability, FactFamily, StrategicFact, ThreatSubject};
use crate::faction::{Capability, Faction};

#[must_use]
#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct VisibleSnapshotId(u8);

impl VisibleSnapshotId {
    pub(crate) const fn for_faction(faction: Faction) -> Self {
        match faction {
            Faction::Zya => Self(1),
            Faction::Arg => Self(2),
        }
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct FactionVisibleSnapshot {
    id: VisibleSnapshotId,
    facts: Vec<StrategicFact>,
}

impl FactionVisibleSnapshot {
    pub(crate) const fn new(id: VisibleSnapshotId, facts: Vec<StrategicFact>) -> Self {
        Self { id, facts }
    }

    pub const fn id(&self) -> VisibleSnapshotId {
        self.id
    }

    pub fn facts(&self) -> &[StrategicFact] {
        &self.facts
    }
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub struct InstitutionView {
    capability: Capability,
    snapshot_id: VisibleSnapshotId,
}

impl InstitutionView {
    pub(crate) const fn new(capability: Capability, snapshot_id: VisibleSnapshotId) -> Self {
        Self {
            capability,
            snapshot_id,
        }
    }

    pub const fn capability(&self) -> Capability {
        self.capability
    }

    pub const fn snapshot_id(&self) -> VisibleSnapshotId {
        self.snapshot_id
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct StrategicPacket {
    faction: Faction,
    policy_version: &'static str,
    visible_snapshot: FactionVisibleSnapshot,
    institution_views: [InstitutionView; 3],
}

impl StrategicPacket {
    pub(crate) fn new(
        faction: Faction,
        policy_version: &'static str,
        facts: Vec<StrategicFact>,
    ) -> Self {
        let snapshot_id = VisibleSnapshotId::for_faction(faction);
        Self {
            faction,
            policy_version,
            visible_snapshot: FactionVisibleSnapshot::new(snapshot_id, facts),
            institution_views: Capability::ALL
                .map(|capability| InstitutionView::new(capability, snapshot_id)),
        }
    }

    pub const fn faction(&self) -> Faction {
        self.faction
    }

    pub fn facts(&self) -> &[StrategicFact] {
        self.visible_snapshot.facts()
    }

    #[must_use]
    pub const fn policy_version(&self) -> &'static str {
        self.policy_version
    }

    pub const fn visible_snapshot_id(&self) -> VisibleSnapshotId {
        self.visible_snapshot.id()
    }

    #[must_use]
    pub const fn institution_views(&self) -> &[InstitutionView; 3] {
        &self.institution_views
    }

    pub fn availability(&self, family: FactFamily) -> FactAvailability {
        self.facts()
            .iter()
            .find(|fact| fact.family() == family)
            .map_or(FactAvailability::Unknown, StrategicFact::availability)
    }

    #[must_use]
    pub fn has_shared_threat(&self, threat: ThreatSubject) -> bool {
        self.facts()
            .iter()
            .any(|fact| fact.family() == FactFamily::Threat && fact.subject() == Some(threat))
    }

    #[must_use]
    pub fn has_observed_threat(&self, threat: ThreatSubject) -> bool {
        self.facts().iter().any(|fact| {
            fact.subject() == Some(threat) && fact.availability() != FactAvailability::Unsupported
        })
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct PairedPackets {
    pub(crate) zya: StrategicPacket,
    pub(crate) arg: StrategicPacket,
}

impl PairedPackets {
    #[must_use]
    pub const fn policy_version(&self) -> &'static str {
        "visibility-v1"
    }

    #[must_use]
    pub const fn packet(&self, faction: Faction) -> &StrategicPacket {
        match faction {
            Faction::Zya => &self.zya,
            Faction::Arg => &self.arg,
        }
    }
}
