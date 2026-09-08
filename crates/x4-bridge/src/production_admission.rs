use observation_application::LifecycleResult;
use observation_domain::SourceScopeId;
use observation_ingest::{
    CarrierIdentity, ControlBody, DispositionBody, ReceiverDisposition, complete_message_digest,
    decode_complete_message,
};

use crate::production_runtime_message::{digest_hex, disposition_name, identity};
use crate::{OperationalHistory, ProductionObservationSession};

#[derive(Debug)]
pub enum AdmitError {
    Decode,
    Identity,
    Lifecycle,
    UnexpectedResult,
    ResponseLoss(SourceScopeId, ReceiverDisposition),
}

impl AdmitError {
    pub const fn reason(&self) -> &'static str {
        match self {
            Self::Decode => "message-decode",
            Self::Identity => "message-identity",
            Self::Lifecycle => "message-lifecycle",
            Self::UnexpectedResult => "message-result",
            Self::ResponseLoss(_, _) => "disposition-response-loss",
        }
    }

    pub const fn response_loss(&self) -> Option<(&SourceScopeId, ReceiverDisposition)> {
        match self {
            Self::ResponseLoss(scope, disposition) => Some((scope, *disposition)),
            _ => None,
        }
    }
}

pub fn finish_error(
    error: &AdmitError,
    history: &mut OperationalHistory,
    active_scope: Option<SourceScopeId>,
) -> Option<SourceScopeId> {
    if let Some((scope, disposition)) = error.response_loss() {
        let _ = history.record(response_loss_state(disposition), error.reason());
        Some(scope.clone())
    } else {
        let _ = history.record("rejected", error.reason());
        active_scope
    }
}

pub fn admit(
    carrier: &CarrierIdentity,
    bytes: &[u8],
    message_limit: usize,
    history: &mut OperationalHistory,
    session: &mut ProductionObservationSession,
    send_response: impl FnOnce(ControlBody) -> Result<(), ()>,
) -> Result<(SourceScopeId, ReceiverDisposition), AdmitError> {
    let decoded = decode_complete_message(bytes, message_limit).map_err(|_| AdmitError::Decode)?;
    let (message_id, section_key, section_revision, scope) =
        identity(&decoded, carrier).map_err(|()| AdmitError::Identity)?;
    history.bind_message(message_id.as_str(), &section_key, section_revision);
    let result = session
        .submit_received(
            carrier.epoch,
            message_id.clone(),
            bytes.to_owned(),
            bytes.len(),
            crate::production_runtime::now(),
        )
        .map_err(|_| AdmitError::Lifecycle)?;
    let LifecycleResult::Disposition(disposition) = result else {
        return Err(AdmitError::UnexpectedResult);
    };
    let body = ControlBody::Disposition(DispositionBody {
        message_id: message_id.as_str().to_owned(),
        section_key,
        section_revision,
        message_digest: digest_hex(complete_message_digest(bytes)),
        disposition: disposition_name(disposition).to_owned(),
    });
    send_response(body).map_err(|()| AdmitError::ResponseLoss(scope.clone(), disposition))?;
    Ok((scope, disposition))
}

pub const fn response_loss_state(disposition: ReceiverDisposition) -> &'static str {
    if matches!(disposition, ReceiverDisposition::Committed) {
        "ambiguous"
    } else {
        "rejected"
    }
}

#[cfg(test)]
#[path = "production_runtime_tests.rs"]
mod tests;
