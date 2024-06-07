use std::collections::HashMap;

use mlua::{
    prelude::{LuaResult, LuaValue},
    Function, IntoLua, Lua, Variadic,
};

use crate::{state::sleep::sleep, value::Value, wrappers::print};

use super::{dm::DmModule, exec::ExecModule, list::ListModule, pointer::PointerModule, LuaModule};

/// Struct that appends the global table with global modules and functions that wouldn't fit any better within a specific module.
///
/// Not a unit struct because the state's internal handle is needed as a parameter to allow passing into lua.
pub struct GlobalModule(pub i32);

impl LuaModule for GlobalModule {
    fn create_items<'lua>(&self, lua: &'lua Lua) -> LuaResult<Vec<(&str, LuaValue<'lua>)>> {
        let id = self.0;
        let id1 = self.0;
        Ok(vec![
            ("dm", (&DmModule as &dyn LuaModule).into_lua(lua)?),
            ("list", (&ListModule as &dyn LuaModule).into_lua(lua)?),
            (
                "loadstring",
                Function::wrap(|lua, code: String| {
                    lua.load(if code.is_empty() { " ".into() } else { code })
                        .into_function()
                })
                .into_lua(lua)?,
            ),
            ("pointer", (&PointerModule as &dyn LuaModule).into_lua(lua)?),
            (
                "sleep",
                unsafe { lua.create_c_function(sleep) }.map(LuaValue::Function)?,
            ),
            (
                "print",
                Function::wrap(move |_, args: Variadic<Value>| print(id1, args)).into_lua(lua)?,
            ),
            ("_exec", (&ExecModule as &dyn LuaModule).into_lua(lua)?),
            ("_state_id", LuaValue::Integer(id)),
        ])
    }
    fn create_metamethods<'lua>(
        &self,
        _: &'lua Lua,
    ) -> LuaResult<HashMap<&'static str, LuaValue<'lua>>> {
        Ok(HashMap::from([("__metatable", LuaValue::Boolean(false))]))
    }
}
