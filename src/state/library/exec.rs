use std::time::Duration;

use mlua::{
    prelude::{LuaError, LuaResult, LuaValue},
    IntoLua, Lua,
};

use crate::state::{
    exec_limit::{get_execution_limit, get_execution_time},
    threads::next_yield_index,
};

use super::LuaModule;

/// Unit struct containing volatile fields related to script execution.
///
/// These would be global fields, but the `mutableGlobals` compiler option does not
/// consider the field itself mutable - only its subfields.
pub struct ExecModule;

impl ExecModule {
    fn exec_limit(lua: &Lua) -> LuaResult<LuaValue> {
        get_execution_limit()
            .as_ref()
            .map(Duration::as_millis)
            .map(i32::try_from)
            .transpose()
            .map_err(LuaError::external)
            .map(|opt| opt.map(LuaValue::Integer))
            .and_then(|opt| opt.into_lua(lua))
    }

    fn exec_time(lua: &Lua) -> LuaResult<LuaValue> {
        get_execution_time()
            .map(i32::try_from)
            .transpose()
            .map_err(LuaError::external)
            .map(|opt| opt.map(LuaValue::Integer))
            .and_then(|opt| opt.into_lua(lua))
    }
}

impl LuaModule for ExecModule {
    fn create_metafield_items(&self) -> Option<super::MetafieldItems> {
        Some(vec![
            ("next_yield_index".into(), Box::new(next_yield_index)),
            ("limit".into(), Box::new(Self::exec_limit)),
            ("time".into(), Box::new(Self::exec_time)),
        ])
    }
}
