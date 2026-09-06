#![cfg(windows)]

use core::{mem::size_of, ptr};

use windows_sys::Win32::{
    Foundation::{CloseHandle, HANDLE},
    Security::{
        CopySid, GetLengthSid, GetTokenInformation, SID_AND_ATTRIBUTES, TOKEN_DUPLICATE,
        TOKEN_GROUPS, TOKEN_QUERY, TokenGroups,
    },
    System::Threading::{GetCurrentProcess, OpenProcessToken},
};

use crate::TransportError;

pub fn current_logon_sid() -> Result<Vec<u8>, TransportError> {
    let mut token: HANDLE = ptr::null_mut();
    // SAFETY: token points to writable handle storage.
    if unsafe {
        OpenProcessToken(
            GetCurrentProcess(),
            TOKEN_QUERY | TOKEN_DUPLICATE,
            &raw mut token,
        )
    } == 0
    {
        return Err(TransportError::Unavailable);
    }
    let result = copy_logon_sid(token);
    // SAFETY: token was returned by OpenProcessToken.
    unsafe { CloseHandle(token) };
    result
}

fn copy_logon_sid(token: HANDLE) -> Result<Vec<u8>, TransportError> {
    let mut needed = 0_u32;
    // SAFETY: null buffer requests the required byte count.
    unsafe { GetTokenInformation(token, TokenGroups, ptr::null_mut(), 0, &raw mut needed) };
    if needed == 0 {
        return Err(TransportError::Unavailable);
    }
    let byte_len = usize::try_from(needed).map_err(|_| TransportError::Unavailable)?;
    let mut groups = vec![0_usize; byte_len.div_ceil(size_of::<usize>())];
    // SAFETY: aligned groups storage owns at least needed writable bytes.
    if unsafe {
        GetTokenInformation(
            token,
            TokenGroups,
            groups.as_mut_ptr().cast(),
            needed,
            &raw mut needed,
        )
    } == 0
    {
        return Err(TransportError::Unavailable);
    }
    let header = groups.as_ptr().cast::<TOKEN_GROUPS>();
    // SAFETY: GetTokenInformation initialized TOKEN_GROUPS and its flexible array.
    let count = unsafe { (*header).GroupCount } as usize;
    // SAFETY: Groups is the first element of count contiguous entries.
    let first = unsafe { ptr::addr_of!((*header).Groups).cast::<SID_AND_ATTRIBUTES>() };
    for index in 0..count {
        // SAFETY: index is bounded by GroupCount in the returned buffer.
        let group = unsafe { &*first.add(index) };
        if let Some(sid) = copy_if_logon(group)? {
            return Ok(sid);
        }
    }
    Err(TransportError::Unavailable)
}

fn copy_if_logon(group: &SID_AND_ATTRIBUTES) -> Result<Option<Vec<u8>>, TransportError> {
    const LOGON_ID: i32 = -1_073_741_824;
    if i32::from_ne_bytes(group.Attributes.to_ne_bytes()) & LOGON_ID != LOGON_ID {
        return Ok(None);
    }
    // SAFETY: group.Sid came from a live TOKEN_GROUPS buffer.
    let sid_len = unsafe { GetLengthSid(group.Sid) };
    let length = usize::try_from(sid_len).map_err(|_| TransportError::Unavailable)?;
    let mut sid = vec![0_u8; length];
    // SAFETY: destination has sid_len bytes and source is a valid SID.
    if unsafe { CopySid(sid_len, sid.as_mut_ptr().cast(), group.Sid) } == 0 {
        Err(TransportError::Unavailable)
    } else {
        Ok(Some(sid))
    }
}
