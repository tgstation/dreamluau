use mlua::{prelude::LuaResult, Lua};

use self::{entrypoint::get_entrypoint_function, traceback::dm_traceback};

pub mod entrypoint;
pub mod traceback;

pub fn prepare_registry_functions(lua: &Lua) -> LuaResult<()> {
    lua.set_named_registry_value("dm_traceback", unsafe {
        lua.create_c_function(dm_traceback)
    }?)?;
    lua.set_named_registry_value("get_entrypoint", unsafe {
        lua.create_c_function(get_entrypoint_function)
    }?)
}
