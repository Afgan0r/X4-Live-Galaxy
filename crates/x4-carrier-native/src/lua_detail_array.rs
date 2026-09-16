use crate::abi_windows::LuaApi;
use core::ffi::c_void;

// Dense arrays only. Raw iteration rejects holes, nonnumeric keys and metatables
// before allocation; every accepted row is copied before the stack is restored.
pub unsafe fn read<T>(
    api: LuaApi,
    state: *mut c_void,
    key: &str,
    limit: usize,
    row: impl FnMut(i32) -> Option<T>,
) -> Option<Vec<T>> {
    unsafe { read_at(api, state, 2, key, limit, row) }
}

pub unsafe fn read_at<T>(
    api: LuaApi,
    state: *mut c_void,
    index: i32,
    key: &str,
    limit: usize,
    mut row: impl FnMut(i32) -> Option<T>,
) -> Option<Vec<T>> {
    let top = unsafe { (api.get_top)(state) };
    unsafe {
        (api.push_string)(state, key.as_ptr().cast(), key.len());
        (api.raw_get)(state, index);
    }
    let result = unsafe { read_table(api, state, top + 1, limit, &mut row) };
    unsafe { (api.set_top)(state, top) };
    result
}

unsafe fn read_table<T>(
    api: LuaApi,
    state: *mut c_void,
    table: i32,
    limit: usize,
    row: &mut impl FnMut(i32) -> Option<T>,
) -> Option<Vec<T>> {
    if unsafe { (api.lua_type)(state, table) } != 5
        || unsafe { (api.get_metatable)(state, table) } != 0
    {
        return None;
    }
    let mut count = 0;
    unsafe { (api.push_nil)(state) };
    while unsafe { (api.next)(state, table) } != 0 {
        let index = unsafe { crate::abi::integer(api, state, -2) }?;
        if index == 0 || index > limit || count >= limit {
            return None;
        }
        count += 1;
        unsafe { (api.set_top)(state, -2) };
    }
    let mut values = Vec::with_capacity(count);
    for index in 1..=count {
        unsafe {
            (api.push_integer)(state, isize::try_from(index).ok()?);
            (api.raw_get)(state, table);
        }
        values.push(row(table + 1)?);
        unsafe { (api.set_top)(state, table) };
    }
    Some(values)
}
