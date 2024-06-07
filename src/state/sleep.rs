use dreamluau_proc_macro::map_statics;
use mlua::lua_State;
use std::{cell::RefCell, os::raw::c_int};

extern "C" {
    fn lua_yield(lua: *mut lua_State, n_results: c_int) -> c_int;
}

thread_local! {
    static SLEEP_FLAG: RefCell<bool> = const { RefCell::new(false) }
}

#[map_statics(mut SLEEP_FLAG)]
pub unsafe extern "C-unwind" fn sleep(lua: *mut lua_State) -> c_int {
    *sleep_flag = true;
    lua_yield(lua, 0)
}

#[map_statics(mut SLEEP_FLAG)]
pub fn take_sleep_flag() -> bool {
    let ret = *sleep_flag;
    *sleep_flag = false;
    ret
}
