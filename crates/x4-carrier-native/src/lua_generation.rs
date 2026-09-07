use core::sync::atomic::{AtomicBool, Ordering};
use std::sync::OnceLock;

use crate::HandleRegistry;

const VARIABLE: &str = "__LIVE_GALAXY_CARRIER_B_NEXT_GENERATION";
static CLAIM: OnceLock<Result<u32, ()>> = OnceLock::new();
static APPLIED: AtomicBool = AtomicBool::new(false);

pub fn initialize(registry: &mut HandleRegistry) -> Result<(), ()> {
    let generation = match CLAIM.get_or_init(claim_process_generation) {
        Ok(value) => *value,
        Err(()) => return Err(()),
    };
    if !APPLIED.swap(true, Ordering::AcqRel) {
        *registry = HandleRegistry::starting_at(generation);
    }
    Ok(())
}

fn claim_process_generation() -> Result<u32, ()> {
    let seed = os_seed()?;
    let (value, next) = claim_values(read_process_value()?.as_deref(), seed)?;
    write_process_value(next)?;
    Ok(value)
}

fn claim_values(raw: Option<&str>, seed: u32) -> Result<(u32, u32), ()> {
    let value = match raw {
        Some(raw) => raw
            .parse::<u32>()
            .ok()
            .filter(|value| *value != 0)
            .ok_or(())?,
        None => (seed != 0).then_some(seed).ok_or(())?,
    };
    Ok((value, value.checked_add(1).ok_or(())?))
}

#[cfg(windows)]
fn os_seed() -> Result<u32, ()> {
    let millis = crate::abi_windows::monotonic_millis().map_err(|_| ())?;
    let mixed = millis ^ u64::from(std::process::id()).rotate_left(17);
    let folded = u32::try_from((mixed ^ (mixed >> 32)) & u64::from(u32::MAX)).map_err(|_| ())?;
    Ok(folded.max(1))
}

#[cfg(not(windows))]
fn os_seed() -> Result<u32, ()> {
    Err(())
}

#[cfg(windows)]
fn read_process_value() -> Result<Option<String>, ()> {
    use windows_sys::Win32::Foundation::{ERROR_ENVVAR_NOT_FOUND, GetLastError};
    let name = wide(VARIABLE);
    let mut buffer = [0_u16; 32];
    let capacity = u32::try_from(buffer.len()).map_err(|_| ())?;
    // SAFETY: name is NUL terminated and buffer owns the advertised capacity.
    let length = unsafe { GetEnvironmentVariableW(name.as_ptr(), buffer.as_mut_ptr(), capacity) };
    if length == 0 {
        // SAFETY: read immediately after the failed Win32 call.
        return if unsafe { GetLastError() } == ERROR_ENVVAR_NOT_FOUND {
            Ok(None)
        } else {
            Err(())
        };
    }
    let length = usize::try_from(length).map_err(|_| ())?;
    if length >= buffer.len() {
        return Err(());
    }
    String::from_utf16(&buffer[..length])
        .map(Some)
        .map_err(|_| ())
}

#[cfg(not(windows))]
fn read_process_value() -> Result<Option<String>, ()> {
    Err(())
}

#[cfg(windows)]
fn write_process_value(value: u32) -> Result<(), ()> {
    let name = wide(VARIABLE);
    let text = wide(&value.to_string());
    // SAFETY: both strings are NUL terminated and live for the call.
    (unsafe { SetEnvironmentVariableW(name.as_ptr(), text.as_ptr()) } != 0)
        .then_some(())
        .ok_or(())
}

#[cfg(not(windows))]
fn write_process_value(_value: u32) -> Result<(), ()> {
    Err(())
}

#[cfg(windows)]
fn wide(value: &str) -> Vec<u16> {
    value.encode_utf16().chain(core::iter::once(0)).collect()
}

#[cfg(windows)]
#[link(name = "kernel32")]
unsafe extern "system" {
    fn GetEnvironmentVariableW(name: *const u16, buffer: *mut u16, size: u32) -> u32;
    fn SetEnvironmentVariableW(name: *const u16, value: *const u16) -> i32;
}

#[cfg(test)]
mod tests {
    use super::claim_values;

    #[test]
    fn stored_generation_is_nonzero_decimal_and_bounded() {
        assert_eq!(claim_values(Some("7"), 9), Ok((7, 8)));
        assert_eq!(claim_values(None, 9), Ok((9, 10)));
        for invalid in ["", "0", "-1", "1.0", "4294967296", " 7"] {
            assert_eq!(claim_values(Some(invalid), 9), Err(()));
        }
        assert_eq!(claim_values(None, 0), Err(()));
        assert_eq!(claim_values(Some("4294967295"), 9), Err(()));
    }
}
