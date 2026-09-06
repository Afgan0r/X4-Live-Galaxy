use core::ffi::{c_char, c_int, c_void};

use crate::InitializerError;

#[cfg(windows)]
use crate::TransportError;

pub type LuaFn = unsafe extern "C" fn(*mut c_void) -> c_int;
pub type CreateTable = unsafe extern "C" fn(*mut c_void, c_int, c_int);
pub type PushClosure = unsafe extern "C" fn(*mut c_void, Option<LuaFn>, c_int);
pub type PushInteger = unsafe extern "C" fn(*mut c_void, isize);
pub type PushString = unsafe extern "C" fn(*mut c_void, *const c_char, usize);
pub type SetField = unsafe extern "C" fn(*mut c_void, c_int, *const c_char);
pub type ToInteger = unsafe extern "C" fn(*mut c_void, c_int) -> isize;
pub type ToString = unsafe extern "C" fn(*mut c_void, c_int, *mut usize) -> *const c_char;

#[derive(Clone, Copy)]
pub struct LuaApi {
    pub create_table: CreateTable,
    pub push_closure: PushClosure,
    pub push_integer: PushInteger,
    pub push_string: PushString,
    pub set_field: SetField,
    pub to_integer: ToInteger,
    pub to_string: ToString,
}

#[cfg(windows)]
pub unsafe fn resolve() -> Result<LuaApi, InitializerError> {
    use windows_sys::Win32::System::LibraryLoader::{GetModuleHandleA, GetProcAddress};

    // SAFETY: null selects the current executable module without changing it.
    let host = unsafe { GetModuleHandleA(core::ptr::null()) };
    if host.is_null() {
        return Err(InitializerError::MissingHostModule);
    }
    macro_rules! get {
        ($name:literal, $ty:ty) => {{
            // SAFETY: the requested symbol name is statically NUL terminated.
            let raw = unsafe { GetProcAddress(host, concat!($name, "\0").as_ptr()) }
                .ok_or(InitializerError::MissingHostSymbol($name))?;
            // SAFETY: Lua 5.1 fixes the signature for each required symbol.
            unsafe { core::mem::transmute::<unsafe extern "system" fn() -> isize, $ty>(raw) }
        }};
    }
    Ok(LuaApi {
        create_table: get!("lua_createtable", CreateTable),
        push_closure: get!("lua_pushcclosure", PushClosure),
        push_integer: get!("lua_pushinteger", PushInteger),
        push_string: get!("lua_pushlstring", PushString),
        set_field: get!("lua_setfield", SetField),
        to_integer: get!("lua_tointeger", ToInteger),
        to_string: get!("lua_tolstring", ToString),
    })
}

#[cfg(windows)]
pub type WorkerEntry = unsafe extern "system" fn(*mut c_void) -> u32;

#[cfg(windows)]
pub fn spawn_worker(entry: WorkerEntry, context: *mut c_void) -> Result<(), TransportError> {
    use windows_sys::Win32::{
        Foundation::CloseHandle,
        System::{
            LibraryLoader::{GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GetModuleHandleExW},
            Threading::CreateThread,
        },
    };
    let mut module = core::ptr::null_mut();
    // SAFETY: FROM_ADDRESS treats the function address as a module address and
    // increments the module reference count for the worker lifetime.
    let retained = unsafe {
        GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS,
            (entry as *const ()).cast::<u16>(),
            &raw mut module,
        )
    };
    if retained == 0 {
        return Err(TransportError::Unavailable);
    }
    // SAFETY: context is a Box transferred to entry; entry owns module release.
    let thread = unsafe {
        CreateThread(
            core::ptr::null(),
            0,
            Some(entry),
            context.cast_const(),
            0,
            core::ptr::null_mut(),
        )
    };
    if thread.is_null() {
        // SAFETY: module was retained above and no worker can release it.
        unsafe { windows_sys::Win32::Foundation::FreeLibrary(module) };
        return Err(TransportError::Unavailable);
    }
    // SAFETY: closing this duplicate does not terminate the running thread.
    unsafe { CloseHandle(thread) };
    Ok(())
}

#[cfg(windows)]
pub fn exit_worker(module_address: WorkerEntry) -> ! {
    use windows_sys::Win32::System::LibraryLoader::{
        GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS, GetModuleHandleExW,
    };
    let mut module = core::ptr::null_mut();
    // SAFETY: the worker already owns one module reference acquired at spawn.
    let found = unsafe {
        GetModuleHandleExW(
            GET_MODULE_HANDLE_EX_FLAG_FROM_ADDRESS
                | windows_sys::Win32::System::LibraryLoader::GET_MODULE_HANDLE_EX_FLAG_UNCHANGED_REFCOUNT,
            (module_address as *const ()).cast::<u16>(),
            &raw mut module,
        )
    };
    if found != 0 {
        // SAFETY: this is the worker's terminal action and consumes its lease.
        unsafe { windows_sys::Win32::System::LibraryLoader::FreeLibraryAndExitThread(module, 0) }
    }
    // SAFETY: failure to recover the module still terminates the worker.
    unsafe { windows_sys::Win32::System::Threading::ExitThread(0) }
}

#[cfg(windows)]
pub fn monotonic_millis() -> Result<u64, TransportError> {
    use windows_sys::Win32::System::Performance::{
        QueryPerformanceCounter, QueryPerformanceFrequency,
    };
    let (mut frequency, mut counter) = (0_i64, 0_i64);
    // SAFETY: both calls initialize their writable i64 outputs.
    if unsafe { QueryPerformanceFrequency(&raw mut frequency) } == 0
        || unsafe { QueryPerformanceCounter(&raw mut counter) } == 0
        || frequency <= 0
        || counter < 0
    {
        return Err(TransportError::Unavailable);
    }
    let scaled = i128::from(counter)
        .checked_mul(1_000)
        .and_then(|value| value.checked_div(i128::from(frequency)))
        .and_then(|value| u64::try_from(value).ok())
        .ok_or(TransportError::Unavailable)?;
    Ok(scaled)
}

#[cfg(not(windows))]
pub unsafe fn resolve() -> Result<LuaApi, InitializerError> {
    Err(InitializerError::MissingHostModule)
}
