use observation_application::LifecycleError;
use observation_domain::EnvelopeDecodeError;

use super::AdmitError;

#[test]
fn decoder_failures_keep_their_actionable_classification() {
    let cases = [
        (
            EnvelopeDecodeError::MessageTooLarge,
            "message-decode-too-large",
        ),
        (
            EnvelopeDecodeError::InvalidShape,
            "message-decode-invalid-shape",
        ),
        (
            EnvelopeDecodeError::UnsupportedVersion,
            "message-decode-unsupported-version",
        ),
        (
            EnvelopeDecodeError::InvalidIdentity,
            "message-decode-invalid-identity",
        ),
        (
            EnvelopeDecodeError::InvalidVersion,
            "message-decode-invalid-version",
        ),
    ];
    for (error, expected) in cases {
        assert_eq!(AdmitError::Decode(error).reason(), expected);
    }
}

#[test]
fn lifecycle_failures_keep_actionable_static_reasons() {
    let cases = [
        (LifecycleError::DecodeRejected, "lifecycle-decode-rejected"),
        (
            LifecycleError::ContextMismatch,
            "lifecycle-context-mismatch",
        ),
        (LifecycleError::SlotInvariant, "lifecycle-slot-invariant"),
        (
            LifecycleError::BlockedAmbiguous,
            "lifecycle-blocked-ambiguous",
        ),
        (LifecycleError::RetainedLimit, "lifecycle-retained-limit"),
        (
            LifecycleError::CompletionRejected,
            "lifecycle-completion-rejected",
        ),
        (
            LifecycleError::AuthorityRejected,
            "lifecycle-authority-rejected",
        ),
        (
            LifecycleError::FinalizationBlocked,
            "lifecycle-finalization-blocked",
        ),
        (
            LifecycleError::RetryNotEligible,
            "lifecycle-retry-not-eligible",
        ),
    ];
    for (error, expected) in cases {
        assert_eq!(
            AdmitError::Production(crate::ProductionError::Lifecycle(error)).reason(),
            expected
        );
    }
}
