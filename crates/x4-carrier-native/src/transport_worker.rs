#![cfg(windows)]

use core::ffi::c_void;
use std::{
    sync::{
        Arc,
        atomic::{Ordering, fence},
        mpsc::{Receiver, SyncSender, TryRecvError},
    },
    time::Duration,
};

use crate::{
    TransportConfig, TransportError,
    abi_windows::{exit_worker, monotonic_millis},
    abi_windows_io::{
        Completion, PendingIo, RawHandle, begin_connect, begin_read, begin_write, cancel,
        close_pipe, poll,
    },
    transport_types::Shared,
};

pub struct WorkerContext {
    pub config: TransportConfig,
    pub pipe: RawHandle,
    pub shared: Arc<Shared>,
    pub data_rx: Receiver<Vec<u8>>,
    pub control_tx: SyncSender<Vec<u8>>,
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
    let Ok(mut connect) = begin_connect(context.pipe) else {
        return terminal(context);
    };
    if connect.is_none() {
        context.shared.connected.store(true, Ordering::Release);
    }
    let mut read = None;
    let mut write = None;
    let mut cancellation_requested = false;
    loop {
        if let Ok(now) = monotonic_millis() {
            context.shared.millis.store(now, Ordering::Release);
            context
                .shared
                .clock_available
                .store(true, Ordering::Release);
        }
        if context.shared.close_requested.load(Ordering::Acquire) && !cancellation_requested {
            cancel_all(context.pipe, [&connect, &read, &write]);
            cancellation_requested = true;
        }
        progress_connect(context, &mut connect);
        progress_write(context, &mut write);
        progress_read(context, &mut read);
        if ready_to_terminate(context, [&connect, &read, &write]) {
            return terminal(context);
        } else if context.shared.connected.load(Ordering::Acquire) {
            start_read(context, &mut read);
            start_write(context, &mut write);
        }
        update_owners(context, [&connect, &read, &write]);
        std::thread::sleep(Duration::from_millis(1));
    }
}

fn ready_to_terminate(context: &WorkerContext, operations: [&Option<PendingIo>; 3]) -> bool {
    context.shared.close_requested.load(Ordering::Acquire)
        && operations.into_iter().all(Option::is_none)
}

fn progress_connect(context: &WorkerContext, operation: &mut Option<PendingIo>) {
    let Some(pending) = operation.as_ref() else {
        return;
    };
    match poll(context.pipe, pending) {
        Completion::Pending => {}
        Completion::Complete(_) => {
            *operation = None;
            context.shared.connected.store(true, Ordering::Release);
        }
        Completion::Cancelled | Completion::Failed => *operation = None,
    }
}

fn start_read(context: &WorkerContext, operation: &mut Option<PendingIo>) {
    if operation.is_none() {
        *operation = begin_read(context.pipe, context.config.max_control_message_bytes).ok();
    }
}

fn progress_read(context: &WorkerContext, operation: &mut Option<PendingIo>) {
    let Some(pending) = operation.as_ref() else {
        return;
    };
    match poll(context.pipe, pending) {
        Completion::Pending => {}
        Completion::Complete(length) => {
            if length <= pending.buffer.len() {
                let _bounded_result = context
                    .control_tx
                    .try_send(pending.buffer[..length].to_vec());
            }
            *operation = None;
        }
        Completion::Cancelled | Completion::Failed => *operation = None,
    }
}

fn start_write(context: &WorkerContext, operation: &mut Option<PendingIo>) {
    if operation.is_some() {
        return;
    }
    match context.data_rx.try_recv() {
        Ok(bytes) => match begin_write(context.pipe, bytes) {
            Ok(pending) => *operation = Some(pending),
            Err(_) => context.shared.data_busy.store(false, Ordering::Release),
        },
        Err(TryRecvError::Empty | TryRecvError::Disconnected) => {}
    }
}

fn progress_write(context: &WorkerContext, operation: &mut Option<PendingIo>) {
    let Some(pending) = operation.as_ref() else {
        return;
    };
    if !matches!(poll(context.pipe, pending), Completion::Pending) {
        *operation = None;
        context.shared.data_busy.store(false, Ordering::Release);
    }
}

fn cancel_all(pipe: RawHandle, operations: [&Option<PendingIo>; 3]) {
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
