#![forbid(unsafe_code)]

mod derive;
mod fact;
mod faction;
mod fingerprint;
mod packet;
mod policy;
mod primitive;
mod primitive_evidence;

pub use derive::{DerivationError, PacketLimits, derive_packets, derive_with_policy};
pub use fact::{FactAvailability, FactFamily, FactReference, StrategicFact, ThreatSubject};
pub use faction::{Capability, Faction, FactionProfile};
pub use fingerprint::{AdmissionInputs, ReplayFingerprint};
pub use packet::{
    FactionVisibleSnapshot, InstitutionView, PairedPackets, StrategicPacket, VisibleSnapshotId,
};
pub use policy::VisibilityPolicy;
pub use primitive::{
    BilateralPosture, PlanningHorizon, PrimitiveOwner, ShadowPrimitive, ShadowPrimitiveError,
    ShadowPrimitiveKind,
};
