use crate::{ObservationPolicyVersion, SectionRevisionId, ShipIdentity, SourceScopeId};

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ShipGroupError {
    DuplicateIdentity(ShipIdentity),
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShipGroupDescriptor {
    source_scope: SourceScopeId,
    core_revision: SectionRevisionId,
    policy_version: ObservationPolicyVersion,
    members: Vec<ShipIdentity>,
}

impl ShipGroupDescriptor {
    pub fn new(
        source_scope: SourceScopeId,
        core_revision: SectionRevisionId,
        policy_version: ObservationPolicyVersion,
        members: impl IntoIterator<Item = ShipIdentity>,
    ) -> Result<Self, ShipGroupError> {
        let members = members.into_iter().collect();
        Ok(Self {
            source_scope,
            core_revision,
            policy_version,
            members,
        })
    }

    pub const fn source_scope(&self) -> &SourceScopeId {
        &self.source_scope
    }
    pub const fn core_revision(&self) -> SectionRevisionId {
        self.core_revision
    }
    pub const fn policy_version(&self) -> ObservationPolicyVersion {
        self.policy_version
    }
    pub fn members(&self) -> &[ShipIdentity] {
        &self.members
    }
}
