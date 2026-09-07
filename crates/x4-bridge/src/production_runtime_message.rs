use observation_domain::{BatchId, CompleteMessage, SourceScopeId};
use observation_ingest::{CarrierIdentity, ReceiverDisposition};
use std::fmt::Write as _;

pub fn identity(
    message: &CompleteMessage,
    expected: &CarrierIdentity,
) -> Result<(BatchId, String, u64, SourceScopeId), ()> {
    let (kind, section, revision, scope, producer, epoch) = match message {
        CompleteMessage::SectionStart(v) => (
            "start",
            &v.section_key,
            v.section_revision,
            v.source_scope.clone(),
            &v.producer_incarnation,
            v.transport_epoch,
        ),
        CompleteMessage::ImmutableBatch(v) => {
            if v.producer_incarnation.as_str() != expected.producer_incarnation
                || v.transport_epoch != expected.epoch
            {
                return Err(());
            }
            return Ok((
                v.batch_id.clone(),
                v.section_key.as_str().to_owned(),
                v.section_revision.get(),
                v.source_scope.clone(),
            ));
        }
        CompleteMessage::SectionCompletion(v) => (
            "complete",
            &v.section_key,
            v.section_revision,
            v.source_scope.clone(),
            &v.producer_incarnation,
            v.transport_epoch,
        ),
        CompleteMessage::Control(_) => return Err(()),
    };
    if producer.as_str() != expected.producer_incarnation || epoch != expected.epoch {
        return Err(());
    }
    let id = BatchId::new(format!(
        "message:{kind}:{}:{}",
        section.as_str(),
        revision.get()
    ))
    .ok_or(())?;
    Ok((id, section.as_str().to_owned(), revision.get(), scope))
}

pub const fn disposition_name(value: ReceiverDisposition) -> &'static str {
    match value {
        ReceiverDisposition::CapacityUnavailable => "capacity_unavailable",
        ReceiverDisposition::Received => "received",
        ReceiverDisposition::Committed => "committed",
        ReceiverDisposition::TimedOutOrSuperseded => "timed_out_or_superseded",
        ReceiverDisposition::StaleEpoch => "stale_epoch",
        ReceiverDisposition::PermanentlyRejected => "permanently_rejected",
        ReceiverDisposition::AmbiguousCommit => "ambiguous_commit",
    }
}

pub fn digest_hex(bytes: [u8; 32]) -> String {
    bytes
        .iter()
        .fold(String::with_capacity(64), |mut text, byte| {
            let _ = write!(text, "{byte:02x}");
            text
        })
}
