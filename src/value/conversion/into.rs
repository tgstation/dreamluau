use super::super::Value;
use crate::cache::userdata::get_or_create_cached_userdata;

use mlua::{
    prelude::{IntoLua, LuaResult, LuaValue},
    Lua, Table,
};

use std::{collections::HashMap, ffi::CString};

use meowtonin::{ByondError, ByondValue, FromByond};

use crate::traits::{AsPrintedExternal, AsPrintedExternalResult};

pub fn convert_to_table(
    lua: &Lua,
    (Value(ref list), deep): (Value, Option<bool>),
) -> LuaResult<Table<'_>> {
    convert_to_table_impl(lua, list, &mut HashMap::new(), deep.unwrap_or(false))
}

fn convert_to_table_impl<'lua>(
    lua: &'lua Lua,
    list: &ByondValue,
    visited: &mut HashMap<ByondValue, Table<'lua>>,
    deep: bool,
) -> LuaResult<Table<'lua>> {
    if !list.is_list() {
        Err(ByondError::NotAList.into_printed_external())
    } else if let Some(table) = visited.get(list) {
        Ok(table.clone())
    } else {
        let table = lua.create_table()?;
        visited.insert(list.clone(), table.clone());
        let mut index = 1;
        list.iter()
            .into_printed_external()
            .and_then(|mut iter| {
                iter.try_for_each(|(key, value)| {
                    let converted_key = if key.is_list() && deep {
                        LuaValue::Table(convert_to_table_impl(lua, &key, visited, deep)?)
                    } else {
                        Value(key.clone()).into_lua(lua)?
                    };
                    let converted_value = if value.is_list() && deep {
                        LuaValue::Table(convert_to_table_impl(lua, &value, visited, deep)?)
                    } else {
                        Value(value.clone()).into_lua(lua)?
                    };
                    table
                        .raw_set(
                            if value.is_null() {
                                LuaValue::Number(index as f64)
                            } else {
                                converted_key.clone()
                            },
                            if value.is_null() {
                                converted_key
                            } else {
                                converted_value
                            },
                        )
                        .map(|_| {
                            index += 1;
                        })
                })
            })
            .map(|_| table)
    }
}

impl<'lua> IntoLua<'lua> for Value {
    fn into_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        if self.0.is_null() {
            return Ok(LuaValue::Nil);
        };
        if self.0.is_number() {
            return Ok(LuaValue::Number(self.0.get_number().unwrap() as f64));
        };
        if self.0.is_string() {
            return CString::from_byond(&self.0)
                .into_printed_external()?
                .into_lua(lua);
        };
        get_or_create_cached_userdata(self, lua)?.into_lua(lua)
    }
}
