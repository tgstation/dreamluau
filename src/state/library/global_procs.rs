use std::collections::HashMap;

use mlua::{
    prelude::{LuaResult, LuaValue},
    Function, IntoLua, Lua, MetaMethod, OwnedFunction, Table,
};

use crate::cache::global_proc::global_proccall_function;

use super::LuaModule;

/// Unit struct for functions wrapping global procs
pub struct GlobalProcsModule;

impl GlobalProcsModule {
    fn index(lua: &Lua, (_, index): (Table, String)) -> LuaResult<OwnedFunction> {
        global_proccall_function(lua, index)
    }
}

impl LuaModule for GlobalProcsModule {
    fn create_metamethods<'lua>(
        &self,
        lua: &'lua Lua,
    ) -> LuaResult<HashMap<&'static str, LuaValue<'lua>>> {
        let index = Function::wrap(Self::index).into_lua(lua)?;
        Ok(HashMap::from([
            (MetaMethod::Index.name(), index),
            ("__metatable", LuaValue::Boolean(false)),
        ]))
    }
}
