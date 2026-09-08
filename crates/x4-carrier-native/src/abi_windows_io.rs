#![cfg(windows)]

use core::ptr;

use windows_sys::Win32::{
    Foundation::{
        CloseHandle, ERROR_IO_INCOMPLETE, ERROR_IO_PENDING, ERROR_OPERATION_ABORTED,
        ERROR_PIPE_CONNECTED, GetLastError, HANDLE, INVALID_HANDLE_VALUE,
    },
    Security::SECURITY_ATTRIBUTES,
    Storage::FileSystem::{FILE_FLAG_OVERLAPPED, PIPE_ACCESS_DUPLEX, ReadFile, WriteFile},
    System::{
        IO::{CancelIoEx, GetOverlappedResult, OVERLAPPED},
        Pipes::{
            ConnectNamedPipe, CreateNamedPipeW, DisconnectNamedPipe, PIPE_READMODE_MESSAGE,
            PIPE_REJECT_REMOTE_CLIENTS, PIPE_TYPE_MESSAGE, PIPE_WAIT,
        },
        Threading::CreateEventW,
    },
};

use crate::TransportError;

pub type RawHandle = HANDLE;

pub struct PendingIo {
    pub buffer: Vec<u8>,
    overlapped: Box<OVERLAPPED>,
    event: HANDLE,
}

pub enum Completion {
    Pending,
    Complete(usize),
    Cancelled,
    Failed,
}

pub fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(core::iter::once(0)).collect()
}

pub fn create_server(
    name: &[u16],
    data_bytes: u32,
    control_bytes: u32,
    security: &SECURITY_ATTRIBUTES,
) -> Result<RawHandle, TransportError> {
    // SAFETY: name is NUL terminated; security points to a live descriptor.
    let handle = unsafe {
        CreateNamedPipeW(
            name.as_ptr(),
            PIPE_ACCESS_DUPLEX | FILE_FLAG_OVERLAPPED,
            PIPE_TYPE_MESSAGE | PIPE_READMODE_MESSAGE | PIPE_WAIT | PIPE_REJECT_REMOTE_CLIENTS,
            1,
            data_bytes,
            control_bytes,
            0,
            security,
        )
    };
    if handle == INVALID_HANDLE_VALUE {
        Err(TransportError::Unavailable)
    } else {
        Ok(handle)
    }
}

pub fn begin_connect(handle: RawHandle) -> Result<Option<PendingIo>, TransportError> {
    let mut operation = PendingIo::empty()?;
    // SAFETY: operation owns stable OVERLAPPED storage through completion.
    let ok = unsafe { ConnectNamedPipe(handle, operation.overlapped.as_mut()) };
    if ok != 0 {
        return Ok(None);
    }
    // SAFETY: called immediately after the failed Win32 operation.
    match unsafe { GetLastError() } {
        ERROR_IO_PENDING => Ok(Some(operation)),
        ERROR_PIPE_CONNECTED => Ok(None),
        _ => Err(TransportError::Unavailable),
    }
}

pub fn begin_read(handle: RawHandle, capacity: usize) -> Result<PendingIo, TransportError> {
    let mut operation = PendingIo::with_capacity(capacity)?;
    let length = u32::try_from(capacity).map_err(|_| TransportError::InvalidConfig)?;
    // SAFETY: buffer and OVERLAPPED storage live until terminal completion.
    let ok = unsafe {
        ReadFile(
            handle,
            operation.buffer.as_mut_ptr(),
            length,
            ptr::null_mut(),
            operation.overlapped.as_mut(),
        )
    };
    operation.started(ok)
}

pub fn begin_write(handle: RawHandle, bytes: Vec<u8>) -> Result<PendingIo, TransportError> {
    let mut operation = PendingIo::from_bytes(bytes)?;
    let length =
        u32::try_from(operation.buffer.len()).map_err(|_| TransportError::InvalidConfig)?;
    // SAFETY: buffer and OVERLAPPED storage live until terminal completion.
    let ok = unsafe {
        WriteFile(
            handle,
            operation.buffer.as_ptr(),
            length,
            ptr::null_mut(),
            operation.overlapped.as_mut(),
        )
    };
    operation.started(ok)
}

pub fn poll(handle: RawHandle, operation: &PendingIo) -> Completion {
    let mut transferred = 0_u32;
    // SAFETY: operation retains the submitted OVERLAPPED and buffer.
    let ok = unsafe {
        GetOverlappedResult(
            handle,
            operation.overlapped.as_ref(),
            &raw mut transferred,
            0,
        )
    };
    if ok != 0 {
        return Completion::Complete(transferred as usize);
    }
    // SAFETY: called immediately after GetOverlappedResult.
    match unsafe { GetLastError() } {
        ERROR_IO_INCOMPLETE | ERROR_IO_PENDING => Completion::Pending,
        ERROR_OPERATION_ABORTED => Completion::Cancelled,
        _ => Completion::Failed,
    }
}

pub fn cancel(handle: RawHandle, operation: &PendingIo) {
    // SAFETY: operation still owns its submitted OVERLAPPED storage.
    unsafe { CancelIoEx(handle, operation.overlapped.as_ref()) };
}

pub fn close_pipe(handle: RawHandle) {
    // SAFETY: worker owns the server handle and calls this after terminal I/O.
    unsafe {
        DisconnectNamedPipe(handle);
        CloseHandle(handle);
    }
}

pub fn disconnect_pipe(handle: RawHandle) {
    // SAFETY: the worker owns a live server pipe handle.
    unsafe { DisconnectNamedPipe(handle) };
}
pub fn close_handle(handle: RawHandle) {
    // SAFETY: caller transfers one owned handle.
    unsafe { CloseHandle(handle) };
}

impl PendingIo {
    fn empty() -> Result<Self, TransportError> {
        Self::from_bytes(Vec::new())
    }
    fn with_capacity(capacity: usize) -> Result<Self, TransportError> {
        Self::from_bytes(vec![0_u8; capacity])
    }
    fn from_bytes(buffer: Vec<u8>) -> Result<Self, TransportError> {
        // SAFETY: null creates an unnamed, process-local manual-reset event.
        let event = unsafe { CreateEventW(ptr::null(), 1, 0, ptr::null()) };
        if event.is_null() {
            return Err(TransportError::Unavailable);
        }
        // SAFETY: zero is the documented initial state for OVERLAPPED.
        let mut overlapped = Box::new(unsafe { core::mem::zeroed::<OVERLAPPED>() });
        overlapped.hEvent = event;
        Ok(Self {
            buffer,
            overlapped,
            event,
        })
    }
    fn started(self, ok: i32) -> Result<Self, TransportError> {
        if ok != 0 {
            return Ok(self);
        }
        // SAFETY: called immediately after ReadFile/WriteFile.
        if unsafe { GetLastError() } == ERROR_IO_PENDING {
            Ok(self)
        } else {
            Err(TransportError::Unavailable)
        }
    }
}
impl Drop for PendingIo {
    fn drop(&mut self) {
        // SAFETY: worker drops an operation only after terminal completion.
        unsafe { CloseHandle(self.event) };
    }
}
