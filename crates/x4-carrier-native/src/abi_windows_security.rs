#![cfg(windows)]

use core::{mem::size_of, ptr};

use windows_sys::Win32::{
    Security::{
        ACCESS_ALLOWED_ACE, ACL, ACL_REVISION, AddAccessAllowedAce, InitializeAcl,
        InitializeSecurityDescriptor, SECURITY_ATTRIBUTES, SECURITY_DESCRIPTOR,
    },
    Storage::FileSystem::{FILE_GENERIC_READ, FILE_GENERIC_WRITE},
};

use crate::{SecurityEvidence, TransportError};

pub struct PipeSecurity {
    descriptor: Box<SECURITY_DESCRIPTOR>,
    acl: Vec<u32>,
    sid: Vec<u8>,
    attributes: SECURITY_ATTRIBUTES,
    current_allowed: bool,
    outside_denied: bool,
}

impl PipeSecurity {
    pub fn current_logon() -> Result<Self, TransportError> {
        let sid = crate::abi_windows_sid::current_logon_sid()?;
        let mut acl = build_acl(&sid)?;
        let acl_ptr = acl.as_mut_ptr().cast::<ACL>();
        // SAFETY: zero is a valid initial byte pattern before initialization.
        let mut descriptor = Box::new(unsafe { core::mem::zeroed::<SECURITY_DESCRIPTOR>() });
        let descriptor_ptr = ptr::from_mut(descriptor.as_mut()).cast();
        // SAFETY: descriptor points to owned writable storage.
        if unsafe { InitializeSecurityDescriptor(descriptor_ptr, 1) } == 0 {
            return Err(TransportError::SecurityPolicyFailed);
        }
        // SAFETY: descriptor and acl stay alive for the PipeSecurity lifetime.
        if unsafe {
            windows_sys::Win32::Security::SetSecurityDescriptorDacl(descriptor_ptr, 1, acl_ptr, 0)
        } == 0
        {
            return Err(TransportError::SecurityPolicyFailed);
        }
        // SAFETY: copied logon SID remains owned by PipeSecurity.
        let sid_ptr = sid.as_ptr().cast_mut().cast();
        if unsafe {
            windows_sys::Win32::Security::SetSecurityDescriptorOwner(descriptor_ptr, sid_ptr, 0)
        } == 0
            || unsafe {
                windows_sys::Win32::Security::SetSecurityDescriptorGroup(descriptor_ptr, sid_ptr, 0)
            } == 0
        {
            return Err(TransportError::SecurityPolicyFailed);
        }
        let attributes = SECURITY_ATTRIBUTES {
            nLength: u32::try_from(size_of::<SECURITY_ATTRIBUTES>())
                .map_err(|_| TransportError::Unavailable)?,
            lpSecurityDescriptor: descriptor_ptr,
            bInheritHandle: 0,
        };
        let (current_allowed, outside_denied) =
            crate::abi_windows_access::verify_access(descriptor_ptr, &sid)
                .map_err(|_| TransportError::AccessProbeFailed)?;
        // SAFETY: owner/group were needed only by AccessCheck. The kernel
        // supplies object ownership while retaining the explicit DACL.
        unsafe {
            windows_sys::Win32::Security::SetSecurityDescriptorOwner(
                descriptor_ptr,
                core::ptr::null_mut(),
                0,
            );
            windows_sys::Win32::Security::SetSecurityDescriptorGroup(
                descriptor_ptr,
                core::ptr::null_mut(),
                0,
            );
        }
        Ok(Self {
            descriptor,
            acl,
            sid,
            attributes,
            current_allowed,
            outside_denied,
        })
    }

    pub const fn attributes(&self) -> &SECURITY_ATTRIBUTES {
        &self.attributes
    }

    pub fn evidence(&self) -> SecurityEvidence {
        SecurityEvidence {
            explicit_descriptor: (!self.attributes.lpSecurityDescriptor.is_null()).into(),
            current_logon_allowed: self.current_allowed.into(),
            outside_logon_denied: self.outside_denied.into(),
            remote_clients_rejected: true.into(),
        }
    }
}

fn build_acl(sid: &[u8]) -> Result<Vec<u32>, TransportError> {
    let entry_bytes = size_of::<ACCESS_ALLOWED_ACE>()
        .checked_sub(size_of::<u32>())
        .and_then(|base| base.checked_add(sid.len()))
        .ok_or(TransportError::Unavailable)?;
    let storage_bytes = size_of::<ACL>()
        .checked_add(entry_bytes)
        .ok_or(TransportError::Unavailable)?;
    let acl_len = u32::try_from(storage_bytes).map_err(|_| TransportError::Unavailable)?;
    let mut acl = vec![0_u32; storage_bytes.div_ceil(size_of::<u32>())];
    let acl_ptr = acl.as_mut_ptr().cast::<ACL>();
    // SAFETY: aligned vector owns at least acl_len writable bytes.
    if unsafe { InitializeAcl(acl_ptr, acl_len, ACL_REVISION) } == 0 {
        return Err(TransportError::SecurityPolicyFailed);
    }
    // SAFETY: sid is valid and acl has room for exactly one ACE.
    if unsafe {
        AddAccessAllowedAce(
            acl_ptr,
            ACL_REVISION,
            FILE_GENERIC_READ | FILE_GENERIC_WRITE,
            sid.as_ptr().cast_mut().cast(),
        )
    } == 0
    {
        return Err(TransportError::SecurityPolicyFailed);
    }
    Ok(acl)
}

impl Drop for PipeSecurity {
    fn drop(&mut self) {
        core::hint::black_box((&self.descriptor, &self.acl, &self.sid));
    }
}
