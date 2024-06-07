use std::ptr;

use meowtonin::{ByondError, ByondResult, ByondValue, ToByond};
use mlua::{
    prelude::{LuaError, LuaValue},
    FromLua, IntoLuaMulti, Lua, ThreadStatus, Variadic,
};

use crate::value::{safe_convert_from_table, ByondObject, ConversionVariant, Value};

use super::{
    exec_limit::{decrement_call_depth, increment_call_depth},
    threads::{get_yielded_thread, pop_front_sleeping_thread, push_yielded_thread, NamedThread},
    usr::{pop_usr, push_usr},
    util::{
        entrypoint::{get_entrypoint, insert_main_chunk, remove_main_chunk},
        traceback::{pop_traceback_func, push_traceback_func},
    },
};

pub fn process_return_values(
    lua: &Lua,
    return_values: Variadic<LuaValue>,
) -> ByondResult<Vec<(&'static str, ByondValue)>> {
    let (values, variants): (Vec<Value>, Vec<ConversionVariant>) = return_values
        .into_iter()
        .map(|value| match value {
            LuaValue::Table(table) => safe_convert_from_table(lua, table).unwrap(),
            LuaValue::Function(ref f) => (
                Value::from(format!(
                    "{}: {:p}",
                    f.info().name.unwrap_or(String::from("anonymous function")),
                    value.to_pointer()
                )),
                ConversionVariant::Function,
            ),
            LuaValue::Thread(_) => (
                Value::from(format!("{:p}", value.to_pointer())),
                ConversionVariant::Thread,
            ),
            LuaValue::UserData(ref ud) if !ud.is::<ByondObject>() => (
                Value::from(format!("{:p}", value.to_pointer())),
                ConversionVariant::Userdata,
            ),
            LuaValue::Error(e) => (Value::from(e.to_string()), ConversionVariant::ErrorAsValue),
            anything_else => match Value::from_lua(anything_else, lua) {
                Ok(v) => (v, ConversionVariant::None),
                Err(e) => (
                    Value(e.to_string().to_byond().unwrap()),
                    ConversionVariant::ConversionError,
                ),
            },
        })
        .collect::<Vec<_>>()
        .into_iter()
        .unzip();
    Ok(vec![
        ("return_values", values.to_byond()?),
        ("variants", variants.to_byond()?),
    ])
}

pub fn run_thread<'lua, A: IntoLuaMulti<'lua> + Clone>(
    lua: &'lua Lua,
    thread: &'lua NamedThread,
    args: A,
) -> ByondResult<ByondValue> {
    let name = thread.name.clone();
    push_traceback_func(lua, &thread.thread).map_err(ByondError::boxed)?;
    push_usr();
    increment_call_depth();
    let result = thread.thread.resume::<A, Variadic<LuaValue>>(args);
    decrement_call_depth();
    pop_usr();
    pop_traceback_func();
    if thread.thread.status() != ThreadStatus::Resumable {
        remove_main_chunk(&get_entrypoint(lua, &thread.thread).unwrap_or(ptr::null()))
    }
    let mut output = match result {
        Ok(return_values) => match thread.thread.status() {
            ThreadStatus::Unresumable => vec![("status", "finished".to_byond().unwrap())],
            ThreadStatus::Resumable => {
                match push_yielded_thread(lua, thread.to_owned()).map_err(ByondError::boxed)? {
                    Some(index) => vec![
                        ("status", "yield".to_byond().unwrap()),
                        ("index", index.to_byond()?),
                    ],
                    None => vec![("status", "sleep".to_byond().unwrap())],
                }
            }
            ThreadStatus::Error => unreachable!("Lua threads that raise an error during execution should not return Ok from resume."),
        }.into_iter().chain(process_return_values(lua, return_values)?).collect(),
        Err(e) => vec![
            ("status", "error".to_byond().unwrap()),
            ("message", e.to_string().to_byond().unwrap()),
        ],
    };
    output.extend([("name", name.to_byond().unwrap())]);
    output.to_byond()
}

pub fn load(lua: &Lua, code: String, name: Option<String>) -> ByondResult<ByondValue> {
    let name = name.unwrap_or("input".into());
    lua.load(code)
        .set_name(&name)
        .into_function()
        .and_then(|func| {
            insert_main_chunk(&func);
            lua.create_thread(func)
        })
        .map_err(ByondError::boxed)
        .and_then(|thread| {
            run_thread(
                lua,
                &NamedThread {
                    name,
                    thread: thread.into_owned(),
                },
                (),
            )
        })
}

pub fn awaken(lua: &Lua) -> ByondResult<ByondValue> {
    pop_front_sleeping_thread(lua)
        .map_err(ByondError::boxed)
        .and_then(|thread| run_thread(lua, &thread, ()))
}

pub fn resume(lua: &Lua, index: usize, args: Vec<Value>) -> ByondResult<ByondValue> {
    get_yielded_thread(lua, index)
        .map_err(ByondError::boxed)
        .and_then(|thread| run_thread(lua, &thread, Variadic::from_iter(args)))
}

pub fn call(lua: &Lua, path: Vec<Value>, args: Vec<Value>) -> ByondResult<ByondValue> {
    let path_length = path.len();
    let mut path = path.into_iter().zip(1..=path_length);
    path.try_fold(LuaValue::Table(lua.globals()), |value, (part, index)| {
        if let LuaValue::Table(tab) = value {
            match tab.raw_get(part)? {
                LuaValue::Table(t) if index < path_length => Ok(LuaValue::Table(t)),
                LuaValue::Function(f) if index == path_length => Ok(LuaValue::Function(f)),
                otherwise => Err(LuaError::external(format!(
                    "invalid function path element at index {}: expected {}, got {}",
                    index,
                    if index == path_length {
                        "function"
                    } else {
                        "table"
                    },
                    otherwise.type_name()
                ))),
            }
        } else {
            unreachable!()
        }
    })
    .and_then(|value| {
        let value_pointer = value.to_pointer();
        if let LuaValue::Function(func) = value {
            let name = func
                .info()
                .name
                .unwrap_or_else(|| format!("Function: {:p}", value_pointer));
            lua.create_thread(func).map(|thread| NamedThread {
                name,
                thread: thread.into_owned(),
            })
        } else {
            unreachable!()
        }
    })
    .map_err(ByondError::boxed)
    .and_then(|thread| run_thread(lua, &thread, Variadic::from_iter(args)))
}
