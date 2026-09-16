use observation_domain::CompleteMessage;
use observation_ingest::decode_complete_message;

use crate::{ProducerError, ProducerFeedback};

pub(super) fn batch_identity(bytes: &[u8], limit: usize) -> Result<String, ProducerError> {
    let CompleteMessage::ImmutableBatch(batch) =
        decode_complete_message(bytes, limit).map_err(|_| ProducerError::InvalidInput)?
    else {
        return Err(ProducerError::InvalidInput);
    };
    Ok(batch.batch_id.as_str().to_owned())
}

pub(super) fn disposition(value: &str) -> Result<ProducerFeedback, ProducerError> {
    match value {
        "capacity_unavailable" => Ok(ProducerFeedback::CapacityUnavailable),
        "received" => Ok(ProducerFeedback::Received),
        "committed" => Ok(ProducerFeedback::Committed),
        "permanently_rejected" => Ok(ProducerFeedback::PermanentlyRejected),
        "ambiguous_commit" => Ok(ProducerFeedback::Ambiguous),
        "stale_epoch" => Ok(ProducerFeedback::StaleEpoch),
        "timed_out_or_superseded" => Ok(ProducerFeedback::TimedOutOrSuperseded),
        _ => Err(ProducerError::InvalidInput),
    }
}

pub(super) fn hex(bytes: [u8; 32]) -> String {
    use core::fmt::Write as _;
    bytes
        .iter()
        .fold(String::with_capacity(64), |mut text, byte| {
            let _ignored = write!(text, "{byte:02x}");
            text
        })
}
