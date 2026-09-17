use crate::abi_windows::LuaApi;
use core::ffi::c_void;

// One shared admission walk precedes owned decoding. Each table entry reserves
// space for typed storage and formatting overhead; strings reserve their actual
// length twice (owned copy plus canonical output). No borrowed pointer escapes.
pub(crate) unsafe fn admit(api: LuaApi, state: *mut c_void, frame_bytes: usize) -> bool {
    let top = unsafe { (api.get_top)(state) };
    let mut remaining = frame_bytes.checked_mul(8);
    let result = unsafe { walk(api, state, 2, 0, &mut remaining) };
    unsafe { (api.set_top)(state, top) };
    result
}

fn charge(remaining: &mut Option<usize>, bytes: usize) -> bool {
    *remaining = remaining.and_then(|value| value.checked_sub(bytes));
    remaining.is_some()
}

unsafe fn walk(
    api: LuaApi,
    state: *mut c_void,
    index: i32,
    depth: usize,
    remaining: &mut Option<usize>,
) -> bool {
    if depth > 5 || !charge(remaining, 128) {
        return false;
    }
    match unsafe { (api.lua_type)(state, index) } {
        4 => {
            let mut length = 0;
            let pointer = unsafe { (api.to_string)(state, index, &raw mut length) };
            !pointer.is_null()
                && length
                    .checked_mul(2)
                    .is_some_and(|bytes| charge(remaining, bytes))
        }
        5 => unsafe { walk_table(api, state, index, depth, remaining) },
        1 | 3 => true,
        _ => false,
    }
}

unsafe fn walk_table(
    api: LuaApi,
    state: *mut c_void,
    index: i32,
    depth: usize,
    remaining: &mut Option<usize>,
) -> bool {
    if unsafe { (api.get_metatable)(state, index) } != 0 {
        return false;
    }
    unsafe { (api.push_nil)(state) };
    while unsafe { (api.next)(state, index) } != 0 {
        let top = unsafe { (api.get_top)(state) };
        if !unsafe { walk(api, state, top - 1, depth + 1, remaining) }
            || !unsafe { walk(api, state, top, depth + 1, remaining) }
        {
            return false;
        }
        unsafe { (api.set_top)(state, top - 1) };
    }
    true
}

#[cfg(test)]
mod tests {
    use super::charge;

    #[test]
    fn nested_arrays_share_one_finite_copy_budget() {
        let mut remaining = Some(512);
        assert!(charge(&mut remaining, 256));
        assert!(charge(&mut remaining, 128));
        assert!(!charge(&mut remaining, 129));
        assert!(!charge(&mut remaining, 0));
    }
}
