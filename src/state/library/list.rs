use std::collections::HashMap;

use crate::{
    traits::AsPrintedExternalResult,
    value::{convert_from_table, convert_to_table, Value},
};

use super::LuaModule;
use meowtonin::{ByondError, ByondValue, ToByond};
use mlua::{
    prelude::{LuaResult, LuaValue},
    Function, IntoLua, Lua, Variadic,
};

/// Unit struct for DM list procs and conversion to/from lua tables
pub struct ListModule;

impl ListModule {
    fn add(_: &Lua, (Value(this), items): (Value, Variadic<Value>)) -> LuaResult<Value> {
        if this.is_list() {
            this.call::<_, _, _, Value>("Add", items.to_vec())
        } else {
            Err(ByondError::NotAList)
        }
        .into_printed_external()
    }

    fn copy(
        _: &Lua,
        (Value(this), start, end): (Value, Option<isize>, Option<isize>),
    ) -> LuaResult<Value> {
        if this.is_list() {
            this.call::<_, _, _, Value>("Copy", [start, end])
        } else {
            Err(ByondError::NotAList)
        }
        .into_printed_external()
    }

    fn cut(
        _: &Lua,
        (Value(this), start, end): (Value, Option<isize>, Option<isize>),
    ) -> LuaResult<Value> {
        if this.is_list() {
            this.call::<_, _, _, Value>("Cut", [start, end])
        } else {
            Err(ByondError::NotAList)
        }
        .into_printed_external()
    }

    fn find(
        _: &Lua,
        (Value(this), Value(elem), start, end): (Value, Value, Option<isize>, Option<isize>),
    ) -> LuaResult<Value> {
        if this.is_list() {
            start.to_byond().and_then(|start| {
                end.to_byond()
                    .and_then(|end| this.call::<_, _, _, Value>("Find", [elem, start, end]))
            })
        } else {
            Err(ByondError::NotAList)
        }
        .into_printed_external()
    }

    fn insert(
        _: &Lua,
        (Value(this), index, items): (Value, isize, Variadic<Value>),
    ) -> LuaResult<Value> {
        if this.is_list() {
            index.to_byond().and_then(|index| {
                let mut args = vec![Value(index)];
                args.extend(items);
                this.call::<_, _, _, Value>("Insert", args)
            })
        } else {
            Err(ByondError::NotAList)
        }
        .into_printed_external()
    }

    fn join(
        _: &Lua,
        (Value(this), glue, start, end): (Value, String, Option<isize>, Option<isize>),
    ) -> LuaResult<Value> {
        if this.is_list() {
            start.to_byond().and_then(|start| {
                end.to_byond().and_then(|end| {
                    this.call::<_, _, _, Value>("Join", [glue.to_byond().unwrap(), start, end])
                })
            })
        } else {
            Err(ByondError::NotAList)
        }
        .into_printed_external()
    }

    fn remove(_: &Lua, (Value(this), items): (Value, Variadic<Value>)) -> LuaResult<Value> {
        if this.is_list() {
            this.call::<_, _, _, Value>("Remove", items)
        } else {
            Err(ByondError::NotAList)
        }
        .into_printed_external()
    }

    fn remove_all(_: &Lua, (Value(this), items): (Value, Variadic<Value>)) -> LuaResult<Value> {
        if this.is_list() {
            this.call::<_, _, _, Value>("RemoveAll", items)
        } else {
            Err(ByondError::NotAList)
        }
        .into_printed_external()
    }

    fn splice(
        _: &Lua,
        (Value(this), start, end, items): (Value, Option<isize>, Option<isize>, Variadic<Value>),
    ) -> LuaResult<Value> {
        if this.is_list() {
            start.to_byond().and_then(|start| {
                end.to_byond().and_then(|end| {
                    let mut args = vec![Value(start), Value(end)];
                    args.extend(items);
                    this.call::<_, _, _, Value>("Splice", args)
                })
            })
        } else {
            Err(ByondError::NotAList)
        }
        .into_printed_external()
    }

    fn swap(_: &Lua, (Value(this), index1, index2): (Value, isize, isize)) -> LuaResult<Value> {
        if this.is_list() {
            this.call::<_, _, _, Value>("Swap", [index1, index2])
        } else {
            Err(ByondError::NotAList)
        }
        .into_printed_external()
    }

    fn filter(_: &Lua, (Value(this), type_): (Value, String)) -> LuaResult<Value> {
        this.iter()
            .map(|iter| {
                iter.filter(|(key, _)| {
                    key.typepath()
                        .map(|typepath| typepath.starts_with(&type_))
                        .unwrap_or(false)
                })
                .collect::<Vec<(ByondValue, ByondValue)>>()
            })
            .and_then(|vec| vec.to_byond())
            .map(Value)
            .into_printed_external()
    }
}

impl LuaModule for ListModule {
    fn create_items<'lua>(&self, lua: &'lua Lua) -> LuaResult<Vec<(&str, LuaValue<'lua>)>> {
        Ok(vec![
            ("add", Function::wrap(Self::add).into_lua(lua)?),
            ("copy", Function::wrap(Self::copy).into_lua(lua)?),
            ("cut", Function::wrap(Self::cut).into_lua(lua)?),
            ("find", Function::wrap(Self::find).into_lua(lua)?),
            ("insert", Function::wrap(Self::insert).into_lua(lua)?),
            ("join", Function::wrap(Self::join).into_lua(lua)?),
            ("remove", Function::wrap(Self::remove).into_lua(lua)?),
            (
                "remove_all",
                Function::wrap(Self::remove_all).into_lua(lua)?,
            ),
            ("splice", Function::wrap(Self::splice).into_lua(lua)?),
            ("swap", Function::wrap(Self::swap).into_lua(lua)?),
            (
                "from_table",
                Function::wrap(convert_from_table).into_lua(lua)?,
            ),
            ("to_table", Function::wrap(convert_to_table).into_lua(lua)?),
            ("filter", Function::wrap(Self::filter).into_lua(lua)?),
        ])
    }

    fn create_metamethods<'lua>(
        &self,
        _: &'lua Lua,
    ) -> LuaResult<HashMap<&'static str, LuaValue<'lua>>> {
        Ok(HashMap::from([("__metatable", LuaValue::Boolean(false))]))
    }
}
