#![cfg(windows)]

use std::sync::atomic::Ordering;

use crate::{
    abi_windows_io::{PendingIo, begin_connect, disconnect_pipe},
    transport_worker::{ConnectionState, WorkerContext, cancel_all, progress_connect},
    transport_worker_io::{drain_for_reconnect, start_read, start_write},
};

pub(super) fn advance_connect(
    context: &WorkerContext,
    state: &mut ConnectionState,
    connect: &mut Option<PendingIo>,
    closing: bool,
) {
    if *state != ConnectionState::Connecting {
        return;
    }
    match progress_connect(context, connect) {
        Ok(true) => *state = ConnectionState::Connected,
        Err(()) if !closing => *state = ConnectionState::Reconnecting,
        Ok(false) | Err(()) => {}
    }
}

pub(super) fn enter_reconnect(
    context: &WorkerContext,
    state: &mut ConnectionState,
    operations: [Option<&PendingIo>; 3],
) {
    *state = ConnectionState::Reconnecting;
    context.shared.connected.store(false, Ordering::Release);
    cancel_all(context.pipe, operations);
}

pub(super) fn advance_reconnect(
    context: &WorkerContext,
    state: &mut ConnectionState,
    connect: &mut Option<PendingIo>,
    read: &mut Option<PendingIo>,
    write: &mut Option<PendingIo>,
    retained_write: &mut Option<Vec<u8>>,
    closing: bool,
) {
    if *state != ConnectionState::Reconnecting {
        return;
    }
    drain_for_reconnect(context, read, write, retained_write);
    if closing || read.is_some() || write.is_some() {
        return;
    }
    disconnect_pipe(context.pipe);
    let Ok(operation) = begin_connect(context.pipe) else {
        return;
    };
    *connect = operation;
    *state = if connect.is_none() {
        context.shared.connected.store(true, Ordering::Release);
        ConnectionState::Connected
    } else {
        ConnectionState::Connecting
    };
}

pub(super) fn start_pending_io(
    context: &WorkerContext,
    state: &mut ConnectionState,
    connect: Option<&PendingIo>,
    read: &mut Option<PendingIo>,
    write: &mut Option<PendingIo>,
    retained_control: &mut Option<Vec<u8>>,
    retained_write: &mut Option<Vec<u8>>,
) {
    let read_failed = retained_control.is_none() && start_read(context, read).is_err();
    let write_failed = start_write(context, write, retained_write).is_err();
    if read_failed || write_failed {
        enter_reconnect(context, state, [connect, read.as_ref(), write.as_ref()]);
    }
}
