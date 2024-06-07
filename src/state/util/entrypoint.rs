use std::{cell::RefCell, collections::HashSet, mem, os::raw::c_void};

use dreamluau_proc_macro::map_statics;
use mlua::{
    ffi::{lua_Debug, lua_getinfo, lua_tothread},
    lua_State,
    prelude::LuaResult,
    Function, IntoLua, Lua,
};

use crate::state::exec_limit::set_privileged_execution;

thread_local! {
    static MAIN_CHUNKS: RefCell<HashSet<*const c_void>> = RefCell::new(HashSet::default());
}

pub unsafe extern "C-unwind" fn get_entrypoint_function(lua: *mut lua_State) -> i32 {
    let lua1 = lua_tothread(lua, -1);
    let mut level = 1;
    let mut ar: lua_Debug = mem::zeroed();
    while lua_getinfo(lua1, level + 1, b"\0".as_ptr() as *const i8, &mut ar) == 1 {
        level += 1
    }
    lua_getinfo(lua1, level, b"f\0".as_ptr() as *const i8, &mut ar)
}

pub fn get_entrypoint<'lua, T: IntoLua<'lua> + Clone>(
    lua: &'lua Lua,
    thread: &T,
) -> LuaResult<*const c_void> {
    set_privileged_execution(true);
    let ret = lua
        .named_registry_value::<Function>("get_entrypoint")
        .and_then(|f| f.call::<T, Function>(thread.clone()))
        .map(|f| f.to_pointer());
    set_privileged_execution(false);
    ret
}

#[map_statics(mut MAIN_CHUNKS)]
pub fn insert_main_chunk(func: &Function) {
    main_chunks.insert(func.to_pointer());
}

#[map_statics(MAIN_CHUNKS)]
pub fn is_main_chunk(f: &*const c_void) -> bool {
    main_chunks.contains(f)
}

#[map_statics(mut MAIN_CHUNKS)]
pub fn remove_main_chunk(f: &*const c_void) {
    main_chunks.remove(f);
}
