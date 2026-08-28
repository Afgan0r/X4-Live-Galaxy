#![forbid(unsafe_code)]

mod derive;
mod fact;
mod faction;
mod packet;
mod policy;

pub use derive::{DerivationError, PacketLimits, derive_packets, derive_with_policy};
pub use fact::{FactAvailability, FactFamily, StrategicFact, ThreatSubject};
pub use faction::{Capability, Faction, FactionProfile};
pub use packet::{
    FactionVisibleSnapshot, InstitutionView, PairedPackets, StrategicPacket, VisibleSnapshotId,
};
pub use policy::VisibilityPolicy;
