use crate::{SenderEvidence, SourceScopeId};

#[must_use]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ShipIdentity(String);

impl ShipIdentity {
    pub fn new(value: impl Into<String>) -> Result<Self, ShipCoreError> {
        let value = value.into();
        if value.is_empty() {
            return Err(ShipCoreError::MissingIdentity);
        }
        if !value.bytes().all(|byte| byte.is_ascii_digit()) || value.starts_with('0') {
            return Err(ShipCoreError::InvalidIdentity);
        }
        Ok(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

macro_rules! required_text {
    ($name:ident, $error:ident) => {
        #[must_use]
        #[derive(Clone, Debug, Eq, Hash, PartialEq)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Result<Self, ShipCoreError> {
                let value = value.into();
                if value.trim().is_empty() {
                    Err(ShipCoreError::$error)
                } else {
                    Ok(Self(value))
                }
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

required_text!(ShipOwner, MissingOwner);
required_text!(ShipType, MissingType);
required_text!(ShipClass, MissingClass);
required_text!(ShipLocation, MissingLocation);

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShipCoreError {
    MissingIdentity,
    InvalidIdentity,
    MissingOwner,
    MissingType,
    MissingClass,
    MissingLocation,
}

#[must_use]
#[derive(Clone, Debug, Eq, PartialEq)]
pub struct ShipCoreRecord {
    source_scope: SourceScopeId,
    identity: ShipIdentity,
    owner: ShipOwner,
    ship_type: ShipType,
    class: ShipClass,
    location: ShipLocation,
    evidence: SenderEvidence,
}

impl ShipCoreRecord {
    pub const fn new(
        source_scope: SourceScopeId,
        identity: ShipIdentity,
        owner: ShipOwner,
        ship_type: ShipType,
        class: ShipClass,
        location: ShipLocation,
        evidence: SenderEvidence,
    ) -> Self {
        Self {
            source_scope,
            identity,
            owner,
            ship_type,
            class,
            location,
            evidence,
        }
    }

    pub const fn source_scope(&self) -> &SourceScopeId {
        &self.source_scope
    }
    pub const fn identity(&self) -> &ShipIdentity {
        &self.identity
    }
    pub const fn owner(&self) -> &ShipOwner {
        &self.owner
    }
    pub const fn ship_type(&self) -> &ShipType {
        &self.ship_type
    }
    pub const fn class(&self) -> &ShipClass {
        &self.class
    }
    pub const fn location(&self) -> &ShipLocation {
        &self.location
    }
    pub const fn evidence(&self) -> &SenderEvidence {
        &self.evidence
    }
}
