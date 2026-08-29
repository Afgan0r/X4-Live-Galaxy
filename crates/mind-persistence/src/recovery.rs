use crate::CheckpointEnvelope;

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum CrashPoint {
    BeforeX4Write,
    AfterX4WriteBeforeAcknowledgement,
    AfterAcknowledgementBeforeProjection,
}

#[derive(Clone, Copy, Debug, Eq, PartialEq)]
pub enum RecoveryDiagnostic {
    Rejected { code: &'static str },
    UnsupportedMigration,
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub struct RecoveryOutcome {
    projection: Option<CheckpointEnvelope>,
    diagnostic: Option<RecoveryDiagnostic>,
    port_write_requested: bool,
}

impl RecoveryOutcome {
    #[must_use]
    pub const fn retained(envelope: CheckpointEnvelope, diagnostic: RecoveryDiagnostic) -> Self {
        Self {
            projection: Some(envelope),
            diagnostic: Some(diagnostic),
            port_write_requested: false,
        }
    }

    pub(super) const fn migrated(envelope: CheckpointEnvelope) -> Self {
        Self {
            projection: Some(envelope),
            diagnostic: None,
            port_write_requested: true,
        }
    }

    pub(super) const fn failed(diagnostic: RecoveryDiagnostic) -> Self {
        Self {
            projection: None,
            diagnostic: Some(diagnostic),
            port_write_requested: false,
        }
    }

    #[must_use]
    pub const fn projection(&self) -> Option<&CheckpointEnvelope> {
        self.projection.as_ref()
    }

    #[must_use]
    pub const fn diagnostic(&self) -> Option<RecoveryDiagnostic> {
        self.diagnostic
    }

    #[must_use]
    pub const fn port_write_requested(&self) -> bool {
        self.port_write_requested
    }
}

#[derive(Clone, Debug, Eq, PartialEq)]
pub enum RecoveryInput {
    Crashed {
        acknowledged: CheckpointEnvelope,
        point: CrashPoint,
    },
    Candidate {
        acknowledged: CheckpointEnvelope,
        candidate: Vec<u8>,
    },
    Migration {
        fallback: Option<CheckpointEnvelope>,
        source: String,
        target: String,
        legacy: Vec<u8>,
    },
}

impl RecoveryInput {
    #[must_use]
    pub fn crashed(
        acknowledged: CheckpointEnvelope,
        _: CheckpointEnvelope,
        point: CrashPoint,
    ) -> Self {
        Self::Crashed {
            acknowledged,
            point,
        }
    }

    #[must_use]
    pub const fn candidate(acknowledged: CheckpointEnvelope, candidate: Vec<u8>) -> Self {
        Self::Candidate {
            acknowledged,
            candidate,
        }
    }

    #[must_use]
    pub fn migration(
        fallback: Option<CheckpointEnvelope>,
        source: &str,
        target: &str,
        legacy: Vec<u8>,
    ) -> Self {
        Self::Migration {
            fallback,
            source: source.into(),
            target: target.into(),
            legacy,
        }
    }
}

#[must_use]
pub fn recover(input: RecoveryInput) -> RecoveryOutcome {
    match input {
        RecoveryInput::Crashed {
            acknowledged,
            point,
        } => recover_crash(acknowledged, point),
        RecoveryInput::Candidate {
            acknowledged,
            candidate,
        } => recover_candidate(acknowledged, &candidate),
        RecoveryInput::Migration {
            fallback,
            source,
            target,
            legacy,
        } => crate::migration::recover_migration(fallback, &source, &target, &legacy),
    }
}

fn recover_crash(acknowledged: CheckpointEnvelope, _: CrashPoint) -> RecoveryOutcome {
    retained_valid(acknowledged, None)
}

fn recover_candidate(acknowledged: CheckpointEnvelope, candidate: &[u8]) -> RecoveryOutcome {
    match CheckpointEnvelope::decode(candidate) {
        Ok(decoded) if decoded == acknowledged => retained_valid(acknowledged, None),
        Ok(decoded) if decoded.sequence() < acknowledged.sequence() => {
            rejected(acknowledged, "stale-cursor")
        }
        Ok(decoded) if decoded.sequence() > acknowledged.sequence() => {
            rejected(acknowledged, "out-of-order-cursor")
        }
        Ok(_) => rejected(acknowledged, "content-collision"),
        Err(_) => rejected(acknowledged, "invalid-envelope"),
    }
}

pub fn retained_valid(
    acknowledged: CheckpointEnvelope,
    diagnostic: Option<RecoveryDiagnostic>,
) -> RecoveryOutcome {
    match acknowledged.encode() {
        Ok(_) => RecoveryOutcome {
            projection: Some(acknowledged),
            diagnostic,
            port_write_requested: false,
        },
        Err(_) => RecoveryOutcome {
            projection: None,
            diagnostic: Some(RecoveryDiagnostic::Rejected {
                code: "invalid-acknowledged",
            }),
            port_write_requested: false,
        },
    }
}

const fn rejected(acknowledged: CheckpointEnvelope, code: &'static str) -> RecoveryOutcome {
    RecoveryOutcome::retained(acknowledged, RecoveryDiagnostic::Rejected { code })
}
