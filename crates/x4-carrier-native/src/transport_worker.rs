#![cfg(windows)]

use core::ffi::c_void;
use std::{
    sync::{
        Arc,
        atomic::{Ordering, fence},
        mpsc::{Receiver, SyncSender},
    },
    time::Duration,
};

use crate::{
    TransportConfig, TransportError,
    abi_windows::{exit_worker, monotonic_millis},
    abi_windows_io::{Completion, PendingIo, RawHandle, begin_connect, cancel, close_pipe, poll},
    transport_types::Shared,
    transport_worker_io::{flush_control, progress_read, progress_write},
    transport_worker_state::{
        advance_connect, advance_reconnect, begin_new_generation, closing, mark_connected,
        start_pending_io,
    },
};

pub struct WorkerContext {
    pub config: TransportConfig,
    pub pipe: RawHandle,
    pub shared: Arc<Shared>,
    pub data_rx: Receiver<Vec<u8>>,
    pub control_tx: SyncSender<(u64, Vec<u8>)>,
}

#[derive(Clone, Copy, Eq, PartialEq)]
pub(super) enum ConnectionState {
    Connecting,
    Connected,
    Reconnecting,
}

pub unsafe extern "system" fn worker_entry(context: *mut c_void) -> u32 {
    // SAFETY: spawn_worker receives exactly one Box<WorkerContext> allocation.
    let context = unsafe { Box::from_raw(context.cast::<WorkerContext>()) };
    run(&context);
    drop(context);
    fence(Ordering::SeqCst);
    exit_worker(worker_entry)
}

fn run(context: &WorkerContext) {
    let (mut connect, mut state) = initial_connection(context);
    let mut read = None;
    let mut write = None;
    let mut retained_control = None;
    let mut retained_write = None;
    let mut cancellation_requested = false;
    loop {
        let closing = closing(
            context,
            [connect.as_ref(), read.as_ref(), write.as_ref()],
            &mut cancellation_requested,
        );
        refresh_clock(context);
        advance_connect(context, &mut state, &mut connect, closing);
        let forced_reconnect = context
            .shared
            .reconnect_requested
            .swap(false, Ordering::AcqRel);
        let io_failed = !forced_reconnect
            && state == ConnectionState::Connected
            && (progress_write(context, &mut write, &mut retained_write)
                || progress_read(context, &mut read, &mut retained_control));
        if (forced_reconnect || io_failed) && state == ConnectionState::Connected && !closing {
            begin_new_generation(
                context,
                &mut state,
                [connect.as_ref(), read.as_ref(), write.as_ref()],
                &mut retained_control,
                &mut retained_write,
            );
        }
        flush_control(context, &mut retained_control);
        advance_reconnect(
            context,
            &mut state,
            &mut connect,
            &mut read,
            &mut write,
            closing,
        );
        if ready_to_terminate(context, [&connect, &read, &write]) {
            return terminal(context);
        }
        if state == ConnectionState::Connected && !closing {
            start_pending_io(
                context,
                &mut state,
                connect.as_ref(),
                &mut read,
                &mut write,
                &mut retained_control,
                &mut retained_write,
            );
        }
        update_owners(context, [&connect, &read, &write]);
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn initial_connection(context: &WorkerContext) -> (Option<PendingIo>, ConnectionState) {
    match begin_connect(context.pipe) {
        Ok(None) => {
            mark_connected(context);
            (None, ConnectionState::Connected)
        }
        Ok(Some(operation)) => (Some(operation), ConnectionState::Connecting),
        Err(_) => (None, ConnectionState::Reconnecting),
    }
}

fn refresh_clock(context: &WorkerContext) {
    let Ok(now) = monotonic_millis() else {
        return;
    };
    context.shared.millis.store(now, Ordering::Release);
    context
        .shared
        .clock_available
        .store(true, Ordering::Release);
}

fn ready_to_terminate(context: &WorkerContext, operations: [&Option<PendingIo>; 3]) -> bool {
    context.shared.close_requested.load(Ordering::Acquire)
        && operations.into_iter().all(Option::is_none)
}

pub(super) fn progress_connect(
    context: &WorkerContext,
    operation: &mut Option<PendingIo>,
) -> Result<bool, ()> {
    let Some(pending) = operation.as_ref() else {
        return Ok(true);
    };
    match poll(context.pipe, pending) {
        Completion::Pending => Ok(false),
        Completion::Complete(_) => {
            *operation = None;
            mark_connected(context);
            Ok(true)
        }
        Completion::Cancelled | Completion::Failed => {
            *operation = None;
            Err(())
        }
    }
}

pub(super) fn cancel_all(pipe: RawHandle, operations: [Option<&PendingIo>; 3]) {
    for operation in operations.into_iter().flatten() {
        cancel(pipe, operation);
    }
}

fn update_owners(context: &WorkerContext, operations: [&Option<PendingIo>; 3]) {
    let count = operations.into_iter().filter(|item| item.is_some()).count();
    context.shared.owners.store(count, Ordering::Release);
}

fn terminal(context: &WorkerContext) {
    close_pipe(context.pipe);
    context.shared.owners.store(0, Ordering::Release);
    context.shared.connected.store(false, Ordering::Release);
    context.shared.closed.store(true, Ordering::Release);
}

pub fn start(context: Box<WorkerContext>) -> Result<(), TransportError> {
    let raw = Box::into_raw(context);
    if crate::abi_windows::spawn_worker(worker_entry, raw.cast()).is_err() {
        // SAFETY: spawn failure leaves ownership with this function.
        let context = unsafe { Box::from_raw(raw) };
        close_pipe(context.pipe);
        drop(context);
        return Err(TransportError::Unavailable);
    }
    Ok(())
}
