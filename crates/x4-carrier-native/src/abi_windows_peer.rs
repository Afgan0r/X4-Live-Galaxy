#![cfg(windows)]

use core::ptr;
use std::time::{Duration, Instant};

use windows_sys::Win32::{
    Foundation::{
        CloseHandle, ERROR_PIPE_BUSY, GENERIC_READ, GENERIC_WRITE, GetLastError,
        INVALID_HANDLE_VALUE,
    },
    Storage::FileSystem::{
        CreateFileW, FILE_SHARE_READ, FILE_SHARE_WRITE, OPEN_EXISTING, ReadFile, WriteFile,
    },
    System::Pipes::{
        PIPE_READMODE_MESSAGE, PeekNamedPipe, SetNamedPipeHandleState, WaitNamedPipeW,
    },
};

use crate::{TransportError, abi_windows_io::RawHandle};

pub fn open_peer(name: &[u16], timeout: Duration) -> Result<RawHandle, TransportError> {
    let deadline = Instant::now() + timeout;
    loop {
        // SAFETY: name is NUL terminated and remaining pointers are optional.
        let handle = unsafe {
            CreateFileW(
                name.as_ptr(),
                GENERIC_READ | GENERIC_WRITE,
                FILE_SHARE_READ | FILE_SHARE_WRITE,
                ptr::null(),
                OPEN_EXISTING,
                0,
                ptr::null_mut(),
            )
        };
        if handle != INVALID_HANDLE_VALUE {
            return configure_peer(handle);
        }
        // SAFETY: read immediately after failed CreateFileW.
        if unsafe { GetLastError() } != ERROR_PIPE_BUSY || Instant::now() >= deadline {
            return Err(TransportError::Unavailable);
        }
        // SAFETY: name is NUL terminated; each wait is one millisecond.
        unsafe { WaitNamedPipeW(name.as_ptr(), 1) };
    }
}

fn configure_peer(handle: RawHandle) -> Result<RawHandle, TransportError> {
    let mode = PIPE_READMODE_MESSAGE;
    // SAFETY: handle is a connected pipe and mode is live.
    if unsafe { SetNamedPipeHandleState(handle, &raw const mode, ptr::null(), ptr::null()) } != 0 {
        return Ok(handle);
    }
    // SAFETY: this branch owns the failed client handle.
    unsafe { CloseHandle(handle) };
    Err(TransportError::Unavailable)
}

pub fn peer_read(handle: RawHandle, capacity: usize) -> Result<Vec<u8>, TransportError> {
    let mut bytes = vec![0_u8; capacity];
    let mut received = 0_u32;
    let length = u32::try_from(capacity).map_err(|_| TransportError::InvalidConfig)?;
    // SAFETY: bytes owns length writable bytes; bridge-only helper may block.
    let ok = unsafe {
        ReadFile(
            handle,
            bytes.as_mut_ptr(),
            length,
            &raw mut received,
            ptr::null_mut(),
        )
    };
    if ok == 0 {
        return Err(TransportError::Unavailable);
    }
    bytes.truncate(received as usize);
    Ok(bytes)
}

pub fn peer_read_timeout(
    handle: RawHandle,
    capacity: usize,
    timeout: Duration,
) -> Result<Option<Vec<u8>>, TransportError> {
    let deadline = Instant::now() + timeout;
    loop {
        let mut available = 0_u32;
        // SAFETY: handle is live and only the available byte count is requested.
        let ok = unsafe {
            PeekNamedPipe(
                handle,
                ptr::null_mut(),
                0,
                ptr::null_mut(),
                &raw mut available,
                ptr::null_mut(),
            )
        };
        if ok == 0 {
            return Err(TransportError::Unavailable);
        }
        if available > 0 {
            return peer_read(handle, capacity).map(Some);
        }
        if Instant::now() >= deadline {
            return Ok(None);
        }
        std::thread::sleep(Duration::from_millis(1));
    }
}

pub fn peer_write(handle: RawHandle, bytes: &[u8]) -> Result<(), TransportError> {
    let mut written = 0_u32;
    let length = u32::try_from(bytes.len()).map_err(|_| TransportError::InvalidConfig)?;
    // SAFETY: bytes is readable for length bytes; bridge-only helper may block.
    let ok = unsafe {
        WriteFile(
            handle,
            bytes.as_ptr(),
            length,
            &raw mut written,
            ptr::null_mut(),
        )
    };
    if ok != 0 && written == length {
        Ok(())
    } else {
        Err(TransportError::Unavailable)
    }
}
