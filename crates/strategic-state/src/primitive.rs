use crate::fact::FactReference;
use crate::packet::StrategicPacket;

const MAX_SHADOW_PRIMITIVES: usize = 4;

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum ShadowPrimitiveKind {
    DefensiveReadiness,
    LogisticsAllocationPriority,
    TerritorialDevelopmentPriority,
    BilateralPostureDisposition,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PrimitiveOwner {
    Defense,
    Economy,
    Territorial,
    Executive,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum PlanningHorizon {
    Immediate,
    NearTerm,
    Sustained,
}

#[derive(Clone, Copy, Debug, Eq, Ord, PartialEq, PartialOrd)]
pub enum BilateralPosture {
    PreserveRelations,
    Deescalate,
    IncreasePressure,
    SeekLimitedCoordination,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum ShadowPrimitiveError {
    UnsupportedKind,
    UnavailableRequiredFact,
    EvidenceLimitExceeded,
    PrimitiveLimitExceeded,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum ShadowPrimitive {
    DefensiveReadiness {
        priority: u8,
        horizon: PlanningHorizon,
        evidence: Vec<FactReference>,
    },
    LogisticsAllocationPriority {
        priority: u8,
        horizon: PlanningHorizon,
        evidence: Vec<FactReference>,
    },
    TerritorialDevelopmentPriority {
        priority: u8,
        horizon: PlanningHorizon,
        evidence: Vec<FactReference>,
    },
    BilateralPostureDisposition {
        priority: u8,
        horizon: PlanningHorizon,
        posture: BilateralPosture,
        evidence: Vec<FactReference>,
    },
}

impl ShadowPrimitive {
    pub fn derive(packet: &StrategicPacket) -> Result<Vec<Self>, ShadowPrimitiveError> {
        let defensive = crate::primitive_evidence::family(packet, crate::FactFamily::Military)?;
        let logistics = crate::primitive_evidence::family(packet, crate::FactFamily::Economic)?;
        let territorial =
            crate::primitive_evidence::family(packet, crate::FactFamily::Territorial)?;
        let executive = crate::primitive_evidence::threat(packet)?;
        let primitives = vec![
            Self::DefensiveReadiness {
                priority: 100,
                horizon: PlanningHorizon::Immediate,
                evidence: defensive,
            },
            Self::LogisticsAllocationPriority {
                priority: 75,
                horizon: PlanningHorizon::NearTerm,
                evidence: logistics,
            },
            Self::TerritorialDevelopmentPriority {
                priority: 50,
                horizon: PlanningHorizon::Sustained,
                evidence: territorial,
            },
            Self::BilateralPostureDisposition {
                priority: 25,
                horizon: PlanningHorizon::NearTerm,
                posture: BilateralPosture::PreserveRelations,
                evidence: executive,
            },
        ];
        if primitives.len() > MAX_SHADOW_PRIMITIVES {
            return Err(ShadowPrimitiveError::PrimitiveLimitExceeded);
        }
        Ok(primitives)
    }

    pub fn reject_unknown_kind(value: &str) -> Result<(), ShadowPrimitiveError> {
        match value {
            "defensive_readiness"
            | "logistics_allocation_priority"
            | "territorial_development_priority"
            | "bilateral_posture_disposition" => Ok(()),
            _ => Err(ShadowPrimitiveError::UnsupportedKind),
        }
    }

    #[must_use]
    pub const fn kind(&self) -> ShadowPrimitiveKind {
        match self {
            Self::DefensiveReadiness { .. } => ShadowPrimitiveKind::DefensiveReadiness,
            Self::LogisticsAllocationPriority { .. } => {
                ShadowPrimitiveKind::LogisticsAllocationPriority
            }
            Self::TerritorialDevelopmentPriority { .. } => {
                ShadowPrimitiveKind::TerritorialDevelopmentPriority
            }
            Self::BilateralPostureDisposition { .. } => {
                ShadowPrimitiveKind::BilateralPostureDisposition
            }
        }
    }

    #[must_use]
    pub const fn owner(&self) -> PrimitiveOwner {
        match self {
            Self::DefensiveReadiness { .. } => PrimitiveOwner::Defense,
            Self::LogisticsAllocationPriority { .. } => PrimitiveOwner::Economy,
            Self::TerritorialDevelopmentPriority { .. } => PrimitiveOwner::Territorial,
            Self::BilateralPostureDisposition { .. } => PrimitiveOwner::Executive,
        }
    }

    #[must_use]
    pub const fn priority(&self) -> u8 {
        match self {
            Self::DefensiveReadiness { priority, .. }
            | Self::LogisticsAllocationPriority { priority, .. }
            | Self::TerritorialDevelopmentPriority { priority, .. }
            | Self::BilateralPostureDisposition { priority, .. } => *priority,
        }
    }

    #[must_use]
    pub const fn horizon(&self) -> PlanningHorizon {
        match self {
            Self::DefensiveReadiness { horizon, .. }
            | Self::LogisticsAllocationPriority { horizon, .. }
            | Self::TerritorialDevelopmentPriority { horizon, .. }
            | Self::BilateralPostureDisposition { horizon, .. } => *horizon,
        }
    }

    #[must_use]
    pub const fn posture(&self) -> Option<BilateralPosture> {
        match self {
            Self::BilateralPostureDisposition { posture, .. } => Some(*posture),
            _ => None,
        }
    }

    #[must_use]
    pub fn evidence(&self) -> &[FactReference] {
        match self {
            Self::DefensiveReadiness { evidence, .. }
            | Self::LogisticsAllocationPriority { evidence, .. }
            | Self::TerritorialDevelopmentPriority { evidence, .. }
            | Self::BilateralPostureDisposition { evidence, .. } => evidence,
        }
    }

    #[must_use]
    pub fn canonical_key(
        &self,
    ) -> (
        ShadowPrimitiveKind,
        PrimitiveOwner,
        u8,
        PlanningHorizon,
        Option<BilateralPosture>,
        Vec<FactReference>,
    ) {
        (
            self.kind(),
            self.owner(),
            self.priority(),
            self.horizon(),
            self.posture(),
            self.evidence().to_vec(),
        )
    }
}
