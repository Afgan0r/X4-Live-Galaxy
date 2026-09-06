use core::ffi::{c_char, c_int, c_void};

use crate::InitializerError;

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

#[cfg(not(windows))]
pub unsafe fn resolve() -> Result<LuaApi, InitializerError> {
    Err(InitializerError::MissingHostModule)
}
