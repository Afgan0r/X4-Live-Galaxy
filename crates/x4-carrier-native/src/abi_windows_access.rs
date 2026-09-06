#![cfg(windows)]

use core::ptr;

use windows_sys::Win32::{
    Foundation::{CloseHandle, HANDLE},
    Security::{
        AccessCheck, DuplicateTokenEx, GENERIC_MAPPING, ImpersonateAnonymousToken, PRIVILEGE_SET,
        PSECURITY_DESCRIPTOR, RevertToSelf, SecurityImpersonation, TOKEN_ALL_ACCESS,
        TokenImpersonation,
    },
    Storage::FileSystem::{
        FILE_ALL_ACCESS, FILE_GENERIC_EXECUTE, FILE_GENERIC_READ, FILE_GENERIC_WRITE,
    },
    System::Threading::{GetCurrentProcess, OpenProcessToken},
};

use crate::TransportError;

pub fn verify_access(
    descriptor: PSECURITY_DESCRIPTOR,
    _logon_sid: &[u8],
) -> Result<(bool, bool), TransportError> {
    let process = process_token()?;
    let current = duplicate(process)?;
    let allowed = check(descriptor, current)?;
    let outside = anonymous_token().is_ok_and(|token| {
        let denied = matches!(check(descriptor, token), Ok(false));
        close(token);
        denied
    });
    close(current);
    close(process);
    Ok((allowed, outside))
}

fn anonymous_token() -> Result<HANDLE, TransportError> {
    use windows_sys::Win32::System::Threading::{GetCurrentThread, OpenThreadToken};
    // SAFETY: impersonation is scoped to this thread and reverted below.
    if unsafe { ImpersonateAnonymousToken(GetCurrentThread()) } == 0 {
        return Err(TransportError::Unavailable);
    }
    let mut token: HANDLE = ptr::null_mut();
    // SAFETY: token points to writable storage for the current thread token.
    let opened =
        unsafe { OpenThreadToken(GetCurrentThread(), TOKEN_ALL_ACCESS, 1, &raw mut token) };
    // SAFETY: balances the successful anonymous impersonation above.
    let reverted = unsafe { RevertToSelf() };
    if opened == 0 || reverted == 0 {
        close(token);
        Err(TransportError::Unavailable)
    } else {
        Ok(token)
    }
}

fn process_token() -> Result<HANDLE, TransportError> {
    let mut token: HANDLE = ptr::null_mut();
    // SAFETY: token points to writable handle storage.
    if unsafe { OpenProcessToken(GetCurrentProcess(), TOKEN_ALL_ACCESS, &raw mut token) } == 0 {
        Err(TransportError::Unavailable)
    } else {
        Ok(token)
    }
}

fn duplicate(token: HANDLE) -> Result<HANDLE, TransportError> {
    let mut duplicate: HANDLE = ptr::null_mut();
    // SAFETY: source token is live and output points to writable storage.
    if unsafe {
        DuplicateTokenEx(
            token,
            TOKEN_ALL_ACCESS,
            ptr::null(),
            SecurityImpersonation,
            TokenImpersonation,
            &raw mut duplicate,
        )
    } == 0
    {
        Err(TransportError::Unavailable)
    } else {
        Ok(duplicate)
    }
}

fn check(descriptor: PSECURITY_DESCRIPTOR, token: HANDLE) -> Result<bool, TransportError> {
    let mapping = GENERIC_MAPPING {
        GenericRead: FILE_GENERIC_READ,
        GenericWrite: FILE_GENERIC_WRITE,
        GenericExecute: FILE_GENERIC_EXECUTE,
        GenericAll: FILE_ALL_ACCESS,
    };
    let mut privilege_storage = vec![PRIVILEGE_SET::default(); 8];
    let privileges = privilege_storage.as_mut_ptr();
    let bytes = privilege_storage
        .len()
        .saturating_mul(core::mem::size_of::<PRIVILEGE_SET>());
    let mut privileges_len = u32::try_from(bytes).map_err(|_| TransportError::Unavailable)?;
    let (mut granted, mut status) = (0_u32, 0_i32);
    // SAFETY: all pointers reference live storage and token is impersonation type.
    let ok = unsafe {
        AccessCheck(
            descriptor,
            token,
            FILE_GENERIC_READ | FILE_GENERIC_WRITE,
            &raw const mapping,
            privileges,
            &raw mut privileges_len,
            &raw mut granted,
            &raw mut status,
        )
    };
    if ok == 0 {
        Err(TransportError::Unavailable)
    } else {
        Ok(status != 0)
    }
}

fn close(handle: HANDLE) {
    if !handle.is_null() {
        // SAFETY: each helper-owned handle is closed exactly once.
        unsafe { CloseHandle(handle) };
    }
}
