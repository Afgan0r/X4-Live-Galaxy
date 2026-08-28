use crate::{CheckpointEnvelope, SCHEMA_VERSION};

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
}

impl RecoveryOutcome {
    #[must_use]
    pub const fn retained(envelope: CheckpointEnvelope, diagnostic: RecoveryDiagnostic) -> Self {
        Self {
            projection: Some(envelope),
            diagnostic: Some(diagnostic),
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
        false
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
        acknowledged: CheckpointEnvelope,
        source: String,
        target: String,
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
    pub fn migration(acknowledged: CheckpointEnvelope, source: &str, target: &str) -> Self {
        Self::Migration {
            acknowledged,
            source: source.into(),
            target: target.into(),
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
            acknowledged,
            source,
            target,
        } => migrate(acknowledged, &source, &target),
    }
}

fn recover_crash(acknowledged: CheckpointEnvelope, _: CrashPoint) -> RecoveryOutcome {
    retained_valid(acknowledged, None)
}

fn recover_candidate(acknowledged: CheckpointEnvelope, candidate: &[u8]) -> RecoveryOutcome {
    match CheckpointEnvelope::decode(candidate) {
        Ok(decoded) if decoded == acknowledged => retained_valid(acknowledged, None),
        Ok(_) => rejected(acknowledged, "content-collision"),
        Err(_) => rejected(acknowledged, "invalid-envelope"),
    }
}

fn migrate(acknowledged: CheckpointEnvelope, source: &str, target: &str) -> RecoveryOutcome {
    if source == SCHEMA_VERSION && target == SCHEMA_VERSION {
        retained_valid(acknowledged, None)
    } else {
        RecoveryOutcome::retained(acknowledged, RecoveryDiagnostic::UnsupportedMigration)
    }
}

fn retained_valid(
    acknowledged: CheckpointEnvelope,
    diagnostic: Option<RecoveryDiagnostic>,
) -> RecoveryOutcome {
    match acknowledged.encode() {
        Ok(_) => RecoveryOutcome {
            projection: Some(acknowledged),
            diagnostic,
        },
        Err(_) => RecoveryOutcome {
            projection: None,
            diagnostic: Some(RecoveryDiagnostic::Rejected {
                code: "invalid-acknowledged",
            }),
        },
    }
}

const fn rejected(acknowledged: CheckpointEnvelope, code: &'static str) -> RecoveryOutcome {
    RecoveryOutcome::retained(acknowledged, RecoveryDiagnostic::Rejected { code })
}
