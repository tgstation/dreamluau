use super::super::{ByondObject, Value};

use mlua::{
    prelude::{FromLua, Lua, LuaError, LuaResult, LuaValue},
    Table,
};

use meowtonin::{ByondResult, ByondValue, FromByond, ToByond};

use std::{collections::HashSet, ffi::CString, os::raw::c_void};

use std::collections::HashMap;

use std::cmp::Ordering;

use crate::traits::AsPrintedExternalResult;

#[derive(Clone, PartialEq)]
pub struct ConversionKVP {
    pub key: ConversionVariant,
    pub value: ConversionVariant,
}

impl ToByond for ConversionKVP {
    fn to_byond(&self) -> ByondResult<ByondValue> {
        vec![
            ("key", self.key.to_byond()?),
            ("value", self.value.to_byond()?),
        ]
        .to_byond()
    }
}

#[derive(Clone, PartialEq)]
pub enum ConversionVariant {
    None,
    ConversionError,
    Function,
    Thread,
    Userdata,
    ErrorAsValue,
    List(Vec<Option<ConversionKVP>>),
}

impl ToByond for ConversionVariant {
    fn to_byond(&self) -> ByondResult<ByondValue> {
        match self {
            Self::None => Ok(ByondValue::null()),
            Self::ConversionError => Ok(ByondValue::new_string("error")),
            Self::Function => Ok(ByondValue::new_string("function")),
            Self::Thread => Ok(ByondValue::new_string("thread")),
            Self::Userdata => Ok(ByondValue::new_string("userdata")),
            Self::ErrorAsValue => Ok(ByondValue::new_string("error_as_value")),
            Self::List(vec) => vec.to_byond(),
        }
    }
}

pub fn convert_from_table(lua: &Lua, table: Table) -> LuaResult<Value> {
    convert_from_table_impl(lua, table, false, &mut HashMap::new()).map(|(v, _)| v)
}

/// Converts a table to a list, ignoring errors, but potentially risking skipping entries.
///
/// Out-of-bounds writes will be skipped.
///
/// Assoc keys that fail to convert will be skipped, and assoc values that fail to convert
/// will be replaced with their error messages.
pub fn safe_convert_from_table(lua: &Lua, table: Table) -> LuaResult<(Value, ConversionVariant)> {
    convert_from_table_impl(lua, table, true, &mut HashMap::new())
}

pub fn convert_from_table_impl(
    lua: &Lua,
    table: Table,
    safe: bool,
    visited: &mut HashMap<*const c_void, Value>,
) -> LuaResult<(Value, ConversionVariant)> {
    let ptr = table.to_pointer();
    if let Some(v) = visited.get(&ptr) {
        Ok((v.clone(), ConversionVariant::None))
    } else {
        ByondValue::new_list()
            .into_printed_external()
            .map(Value)
            .and_then(|Value(mut list)| {
                visited.insert(ptr, Value(list.clone()));
                let mut variants: Vec<Option<ConversionKVP>> = vec![];
                let mut key_error_messages: HashSet<String> = HashSet::new();
                let mut pairs = table
                    .pairs::<LuaValue, LuaValue>()
                    .map(Result::unwrap)
                    .collect::<Vec<_>>();
                // Ensure integers (and floats that are exactly equal to integers) come before floats, which come before everything else.
                // BYOND does not allow floats as list indices, and writing to a list by integer index after writing to it by assoc index
                // risks clobbering an assoc entry.
                pairs.sort_by(|(key1, _), (key2, _)| match (key1, key2) {
                    (LuaValue::Integer(i1), LuaValue::Integer(i2)) => i1.cmp(i2),
                    (LuaValue::Integer(i), LuaValue::Number(n))
                        if n.fract() == 0.0 && n < &(f32::MAX as f64) =>
                    {
                        (*i as f64).partial_cmp(n).unwrap_or(Ordering::Less)
                    }
                    (LuaValue::Number(n), LuaValue::Integer(i))
                        if n.fract() == 0.0 && n < &(f32::MAX as f64) =>
                    {
                        n.partial_cmp(&(*i as f64)).unwrap_or(Ordering::Greater)
                    }
                    (LuaValue::Number(n1), LuaValue::Number(n2)) => n1.total_cmp(n2),
                    (LuaValue::Integer(_), _) => Ordering::Less,
                    (LuaValue::Number(_), _) => Ordering::Less,
                    (_, LuaValue::Integer(_)) => Ordering::Greater,
                    (_, LuaValue::Number(_)) => Ordering::Greater,
                    _ => Ordering::Equal,
                });
                let max_integer_index = pairs.iter().fold(0, |max, (key, _)| match key {
                    LuaValue::Integer(i) => max.max(*i),
                    LuaValue::Number(n) if n.fract() == 0.0 && n < &(f32::MAX as f64) => {
                        max.max(*n as i32)
                    }
                    _ => max,
                });
                // Set the length of the list to the largest integer key of the table.
                // Writing to a list by integer index requires that the index be in the interval [1,len]
                match list.write_var("len", max_integer_index) {
                    Ok(()) => {
                        if safe {
                            variants.resize_with(max_integer_index as usize, || None)
                        }
                    }
                    Err(_) if safe => (),
                    Err(e) => return Err(e).into_printed_external(),
                };
                pairs
                    .into_iter()
                    .try_fold(Value(list), |Value(mut list), (key, value)| {
                        let (key, key_variant) = match key {
                            LuaValue::Table(t) => convert_from_table_impl(lua, t, safe, visited)?,
                            LuaValue::Function(ref f) if safe => (
                                Value::from(format!(
                                    "{}: {:p}",
                                    f.info().name.unwrap_or(String::from("anonymous function")),
                                    key.to_pointer()
                                )),
                                ConversionVariant::Function,
                            ),
                            LuaValue::Thread(_) if safe => (
                                Value::from(format!("{:p}", key.to_pointer())),
                                ConversionVariant::Thread,
                            ),
                            LuaValue::UserData(ref ud) if !ud.is::<ByondObject>() && safe => (
                                Value::from(format!("{:p}", key.to_pointer())),
                                ConversionVariant::Userdata,
                            ),
                            LuaValue::Error(e) if safe => {
                                (Value::from(e.to_string()), ConversionVariant::ErrorAsValue)
                            }
                            anything_else => match Value::from_lua(anything_else, lua) {
                                Ok(v) => (v, ConversionVariant::None),
                                Err(e) if safe => {
                                    let mut err_string = e.to_string();
                                    while key_error_messages.contains(&err_string) {
                                        err_string.push(' ');
                                    }
                                    key_error_messages.insert(err_string.clone());
                                    (
                                        Value(ByondValue::new_string(err_string)),
                                        ConversionVariant::ConversionError,
                                    )
                                }
                                Err(e) => return Err(e).into_printed_external(),
                            },
                        };
                        let (value, value_variant) = match value {
                            LuaValue::Table(t) => convert_from_table_impl(lua, t, safe, visited)?,
                            LuaValue::Function(ref f) if safe => (
                                Value::from(format!(
                                    "{}: {:p}",
                                    f.info().name.unwrap_or(String::from("anonymous function")),
                                    value.to_pointer()
                                )),
                                ConversionVariant::Function,
                            ),
                            LuaValue::Thread(_) if safe => (
                                Value::from(format!("{:p}", value.to_pointer())),
                                ConversionVariant::Thread,
                            ),
                            LuaValue::UserData(ref ud) if !ud.is::<ByondObject>() && safe => (
                                Value::from(format!("{:p}", value.to_pointer())),
                                ConversionVariant::Userdata,
                            ),
                            LuaValue::Error(e) if safe => {
                                (Value::from(e.to_string()), ConversionVariant::ErrorAsValue)
                            }
                            anything_else => match Value::from_lua(anything_else, lua) {
                                Ok(v) => (v, ConversionVariant::None),
                                Err(e) if safe => (
                                    Value(ByondValue::new_string(e.to_string())),
                                    ConversionVariant::ConversionError,
                                ),
                                Err(e) => return Err(e).into_printed_external(),
                            },
                        };
                        list.write_list_index(key.clone(), value)
                            .or_else(|e| {
                                if safe {
                                    Ok(())
                                } else {
                                    Err(e).into_printed_external()
                                }
                            })
                            .map(|()| {
                                if safe {
                                    let kvp = if key.0.is_number() {
                                        ConversionKVP {
                                            key: value_variant,
                                            value: ConversionVariant::None,
                                        }
                                    } else {
                                        ConversionKVP {
                                            key: key_variant,
                                            value: value_variant,
                                        }
                                    };
                                    let can_be_none = kvp
                                        == ConversionKVP {
                                            key: ConversionVariant::None,
                                            value: ConversionVariant::None,
                                        };
                                    if let Ok(i) = usize::from_byond(&key.0) {
                                        if !can_be_none {
                                            variants[i - 1].replace(kvp);
                                        }
                                    } else {
                                        variants.push((!can_be_none).then_some(kvp))
                                    }
                                };
                                Value(list)
                            })
                    })
                    .map(|list| (list, ConversionVariant::List(variants)))
            })
            .or_else(|e| {
                if safe {
                    Ok((
                        Value(ByondValue::new_string(e.to_string())),
                        ConversionVariant::ConversionError,
                    ))
                } else {
                    Err(e)
                }
            })
    }
}

impl<'lua> FromLua<'lua> for Value {
    fn from_lua(value: LuaValue<'lua>, lua: &'lua Lua) -> LuaResult<Self> {
        match value {
            LuaValue::Nil => Ok(Self(ByondValue::null())),
            LuaValue::Boolean(b) => Ok(Self(b.to_byond().unwrap())),
            LuaValue::Integer(n) => n.to_byond().map(Self).into_printed_external(),
            LuaValue::Number(n) => (n as f32).to_byond().map(Self).into_printed_external(),
            LuaValue::String(s) => CString::new(s.as_bytes())
                .map(ByondValue::new_string)
                .map(Self)
                .map_err(LuaError::external),
            LuaValue::UserData(u) if u.is::<ByondObject>() => {
                u.borrow::<ByondObject>().map(|r| Value(r.0.clone()))
            }
            LuaValue::Table(t) => convert_from_table(lua, t),
            LuaValue::Vector(v) => [v.x(), v.y(), v.z()]
                .to_byond()
                .map(Self)
                .into_printed_external(),
            LuaValue::Error(e) => Ok(Self(e.to_string().to_byond().unwrap())),
            anything_else => Err(LuaError::FromLuaConversionError {
                from: anything_else.type_name(),
                to: "BYOND value",
                message: Some(String::from("Unsupported value type")),
            }),
        }
    }
}
