use std::collections::HashMap;

use crate::{traits::AsPrintedExternalResult, types::type_name_for_obj, value::Value};

use super::LuaModule;

use mlua::{
    prelude::{LuaError, LuaResult, LuaValue},
    Function, IntoLua, Lua,
};

/// Unit struct for reading/writing DM pointers
pub struct PointerModule;

impl PointerModule {
    fn read(_: &Lua, Value(ref value): Value) -> LuaResult<Value> {
        if value.is_ref() {
            value.read_pointer::<Value>().into_printed_external()
        } else {
            Err(LuaError::external(format!(
                "expected pointer, got {}",
                type_name_for_obj(value)
            )))
        }
    }

    fn write(_: &Lua, (Value(ref mut pointer), Value(value)): (Value, Value)) -> LuaResult<()> {
        if pointer.is_ref() {
            pointer.write_pointer(value).into_printed_external()
        } else {
            Err(LuaError::external(format!(
                "expected pointer, got {}",
                type_name_for_obj(pointer)
            )))
        }
    }

    fn unwrap(_: &Lua, Value(value): Value) -> LuaResult<Value> {
        if value.is_ref() {
            value.read_pointer::<Value>().into_printed_external()
        } else {
            Ok(Value(value))
        }
    }
}

impl LuaModule for PointerModule {
    fn create_items<'lua>(&self, lua: &'lua Lua) -> LuaResult<Vec<(&str, LuaValue<'lua>)>> {
        let read = Function::wrap(Self::read).into_lua(lua)?;
        let write = Function::wrap(Self::write).into_lua(lua)?;
        let unwrap = Function::wrap(Self::unwrap).into_lua(lua)?;
        Ok(vec![("read", read), ("write", write), ("unwrap", unwrap)])
    }

    fn create_metamethods<'lua>(
        &self,
        _: &'lua Lua,
    ) -> LuaResult<HashMap<&'static str, LuaValue<'lua>>> {
        Ok(HashMap::from([("__metatable", LuaValue::Boolean(false))]))
    }
}
