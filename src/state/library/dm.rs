use std::collections::HashMap;

use mlua::{
    prelude::{Lua, LuaResult, LuaValue},
    Function, IntoLua, Variadic,
};

use crate::{
    helpers::{GLOBALS, WORLD},
    state::usr::peek_usr,
    traits::AsPrintedExternalResult,
    value::{ByondObject, Value},
    wrappers::{wrapped_new, wrapped_read_var},
};

use super::{global_procs::GlobalProcsModule, LuaModule, MetafieldItems};

/// Unit struct implementing basic dm-related values and operations
pub struct DmModule;

impl DmModule {
    #[allow(clippy::new_ret_no_self)] // It's called new because it makes a new DM value.
    fn new(_: &Lua, (type_, args): (String, Variadic<Value>)) -> LuaResult<Value> {
        wrapped_new(type_, args.to_vec()).into_printed_external()
    }

    fn is_valid_ref(_: &Lua, value: LuaValue) -> LuaResult<bool> {
        Ok(match value {
            LuaValue::UserData(ud) => ud
                .borrow::<ByondObject>()
                .ok()
                .map(|obj| obj.0.clone().test_ref().is_some())
                .unwrap_or(false),
            _ => false,
        })
    }

    fn usr(lua: &Lua) -> LuaResult<LuaValue> {
        peek_usr().map(Value).into_lua(lua)
    }

    fn get_var(_: &Lua, (Value(ref src), var): (Value, String)) -> LuaResult<Value> {
        wrapped_read_var(src, var).into_printed_external()
    }
}

impl LuaModule for DmModule {
    fn create_items<'lua>(&self, lua: &'lua Lua) -> LuaResult<Vec<(&str, LuaValue<'lua>)>> {
        Ok(vec![
            ("world", Value(WORLD.clone()).into_lua(lua)?),
            ("global_vars", Value(GLOBALS.clone()).into_lua(lua)?),
            (
                "global_procs",
                (&GlobalProcsModule as &dyn LuaModule).into_lua(lua)?,
            ),
            ("get_var", Function::wrap(Self::get_var).into_lua(lua)?),
            ("new", Function::wrap(Self::new).into_lua(lua)?),
            (
                "is_valid_ref",
                Function::wrap(Self::is_valid_ref).into_lua(lua)?,
            ),
        ])
    }

    fn create_metamethods<'lua>(
        &self,
        _: &'lua Lua,
    ) -> LuaResult<HashMap<&'static str, LuaValue<'lua>>> {
        Ok(HashMap::from([("__metatable", LuaValue::Boolean(false))]))
    }

    fn create_metafield_items(&self) -> Option<MetafieldItems> {
        Some(vec![("usr".into(), Box::new(Self::usr))])
    }
}
