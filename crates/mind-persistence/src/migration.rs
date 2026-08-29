use crate::{
    CheckpointEnvelope, RecoveryDiagnostic, RecoveryOutcome, SCHEMA_VERSION,
    recovery::retained_valid,
};

pub fn recover_migration(
    fallback: Option<CheckpointEnvelope>,
    source: &str,
    target: &str,
    legacy: &[u8],
) -> RecoveryOutcome {
    match (source, target) {
        (SCHEMA_VERSION, SCHEMA_VERSION) => fallback.map_or_else(
            || rejected_without_fallback("missing-fallback"),
            |value| retained_valid(value, None),
        ),
        ("mind-checkpoint-v0", SCHEMA_VERSION) => crate::legacy::decode_and_convert(legacy)
            .map_or_else(
                |_| retain_or_reject(fallback, "invalid-legacy"),
                RecoveryOutcome::migrated,
            ),
        _ => fallback.map_or_else(
            || RecoveryOutcome::failed(RecoveryDiagnostic::UnsupportedMigration),
            |value| RecoveryOutcome::retained(value, RecoveryDiagnostic::UnsupportedMigration),
        ),
    }
}

fn retain_or_reject(fallback: Option<CheckpointEnvelope>, code: &'static str) -> RecoveryOutcome {
    fallback.map_or_else(
        || rejected_without_fallback(code),
        |value| RecoveryOutcome::retained(value, RecoveryDiagnostic::Rejected { code }),
    )
}

const fn rejected_without_fallback(code: &'static str) -> RecoveryOutcome {
    RecoveryOutcome::failed(RecoveryDiagnostic::Rejected { code })
}
