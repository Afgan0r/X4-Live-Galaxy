#![forbid(unsafe_code)]

mod derive;
mod fact;
mod faction;

pub use derive::{DerivationError, PacketLimits, PairedPackets, StrategicPacket, derive_packets};
pub use fact::{FactAvailability, FactFamily, StrategicFact, ThreatSubject};
pub use faction::Faction;
