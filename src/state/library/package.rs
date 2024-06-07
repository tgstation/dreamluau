use std::env;

use mlua::{
    prelude::{LuaResult, LuaValue},
    IntoLua, Lua,
};

use super::LuaModule;

/// Unit struct that replaces mlua's static package path with a metafield that returns the appropriate environment var.
pub struct PackageModule;

impl PackageModule {
    fn get_path_env_var(lua: &Lua) -> LuaResult<LuaValue> {
        env::var("LUAU_PATH")
            .or_else(|_| env::var("LUA_PATH"))
            .unwrap_or(String::from("?.luau;?.lua"))
            .into_lua(lua)
    }
}

impl LuaModule for PackageModule {
    fn create_items<'lua>(&self, _: &'lua Lua) -> LuaResult<Vec<(&str, LuaValue<'lua>)>> {
        Ok(vec![("path", LuaValue::Nil)])
    }

    fn create_metafield_items(&self) -> Option<super::MetafieldItems> {
        Some(vec![("path".into(), Box::new(Self::get_path_env_var))])
    }
}
