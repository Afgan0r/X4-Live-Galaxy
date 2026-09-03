use std::num::NonZeroU64;

macro_rules! string_identity {
    ($name:ident) => {
        #[must_use]
        #[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(String);

        impl $name {
            pub fn new(value: impl Into<String>) -> Option<Self> {
                let value = value.into();
                (!value.trim().is_empty()).then_some(Self(value))
            }

            #[must_use]
            pub fn as_str(&self) -> &str {
                &self.0
            }
        }
    };
}

macro_rules! non_zero_identity {
    ($name:ident) => {
        #[must_use]
        #[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
        pub struct $name(NonZeroU64);

        impl $name {
            #[must_use]
            pub const fn new(value: u64) -> Option<Self> {
                match NonZeroU64::new(value) {
                    Some(value) => Some(Self(value)),
                    None => None,
                }
            }

            #[must_use]
            pub const fn get(self) -> u64 {
                self.0.get()
            }
        }
    };
}

string_identity!(SourceScopeId);
string_identity!(ProducerIncarnationId);
string_identity!(SectionKey);
string_identity!(BatchId);
string_identity!(RecordId);
string_identity!(DecisionSnapshotId);

non_zero_identity!(TransportEpoch);
non_zero_identity!(SectionRevisionId);
non_zero_identity!(ObservationSchemaVersion);
non_zero_identity!(ObservationPolicyVersion);
non_zero_identity!(CanonicalizationVersion);
non_zero_identity!(DigestAlgorithmVersion);

#[must_use]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EntityId(String);

impl EntityId {
    pub fn new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        (!value.trim().is_empty()).then_some(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[must_use]
#[derive(Clone, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct EventId(String);

impl EventId {
    pub fn new(value: impl Into<String>) -> Option<Self> {
        let value = value.into();
        (!value.trim().is_empty()).then_some(Self(value))
    }

    #[must_use]
    pub fn as_str(&self) -> &str {
        &self.0
    }
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, PartialEq)]
pub enum ObservationSource {
    X4Runtime,
}

impl ObservationSource {
    pub const fn x4_runtime() -> Self {
        Self::X4Runtime
    }
}

#[must_use]
#[derive(Clone, Copy, Debug, Eq, Hash, Ord, PartialEq, PartialOrd)]
pub struct ObservationTime(u64);

impl ObservationTime {
    pub const fn from_unix_millis(value: u64) -> Self {
        Self(value)
    }
    #[must_use]
    pub const fn unix_millis(self) -> u64 {
        self.0
    }
}

non_zero_identity!(ObservationVersion);
