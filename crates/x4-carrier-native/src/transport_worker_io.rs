#![cfg(windows)]

use std::sync::{atomic::Ordering, mpsc::TrySendError};

use crate::{
    abi_windows_io::{Completion, PendingIo, begin_read, begin_write, poll},
    transport_worker::WorkerContext,
};

pub(super) fn start_read(
    context: &WorkerContext,
    operation: &mut Option<PendingIo>,
) -> Result<(), ()> {
    if operation.is_none() {
        *operation = Some(
            begin_read(context.pipe, context.config.max_control_message_bytes).map_err(|_| ())?,
        );
    }
    Ok(())
}

pub(super) fn progress_read(
    context: &WorkerContext,
    operation: &mut Option<PendingIo>,
    retained: &mut Option<Vec<u8>>,
) -> bool {
    let Some(pending) = operation.as_ref() else {
        return false;
    };
    match poll(context.pipe, pending) {
        Completion::Pending => false,
        Completion::Complete(length) => {
            if length <= pending.buffer.len() {
                *retained = Some(pending.buffer[..length].to_vec());
            }
            *operation = None;
            false
        }
        Completion::Cancelled | Completion::Failed => {
            *operation = None;
            true
        }
    }
}

pub(super) fn start_write(
    context: &WorkerContext,
    operation: &mut Option<PendingIo>,
    retained: &mut Option<Vec<u8>>,
) -> Result<(), ()> {
    if operation.is_some() {
        return Ok(());
    }
    let bytes = retained.take().or_else(|| context.data_rx.try_recv().ok());
    if let Some(bytes) = bytes {
        let retry = bytes.clone();
        let Ok(pending) = begin_write(context.pipe, bytes) else {
            *retained = Some(retry);
            return Err(());
        };
        *operation = Some(pending);
    }
    Ok(())
}

pub(super) fn progress_write(
    context: &WorkerContext,
    operation: &mut Option<PendingIo>,
    retained: &mut Option<Vec<u8>>,
) -> bool {
    let Some(pending) = operation.as_ref() else {
        return false;
    };
    match poll(context.pipe, pending) {
        Completion::Pending => false,
        Completion::Complete(length) if length == pending.buffer.len() => {
            *operation = None;
            context.shared.data_busy.store(false, Ordering::Release);
            false
        }
        Completion::Complete(_) | Completion::Cancelled | Completion::Failed => {
            *retained = operation.take().map(|value| value.buffer.clone());
            true
        }
    }
}

pub(super) fn flush_control(context: &WorkerContext, retained: &mut Option<Vec<u8>>) {
    let Some(bytes) = retained.take() else {
        return;
    };
    let generation = context.shared.connection_generation.load(Ordering::Acquire);
    match context.control_tx.try_send((generation, bytes)) {
        Ok(()) => {}
        Err(TrySendError::Full((_, bytes)) | TrySendError::Disconnected((_, bytes))) => {
            *retained = Some(bytes);
        }
    }
}

pub(super) fn drain_for_reconnect(
    context: &WorkerContext,
    read: &mut Option<PendingIo>,
    write: &mut Option<PendingIo>,
) {
    if read
        .as_ref()
        .is_some_and(|pending| !matches!(poll(context.pipe, pending), Completion::Pending))
    {
        *read = None;
    }
    if write
        .as_ref()
        .is_some_and(|pending| !matches!(poll(context.pipe, pending), Completion::Pending))
    {
        *write = None;
    }
}
