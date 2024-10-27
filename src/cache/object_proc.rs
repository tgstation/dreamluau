use std::{
    collections::HashMap,
    ops::{Deref, DerefMut},
};

use mlua::{prelude::LuaResult, Function, Lua, OwnedFunction, Variadic};

use crate::{traits::AsPrintedExternalResult, wrappers::wrapped_object_call};

use crate::value::Value;

/// Wrapper struct to differentiate cached object functions from cached global functions, as internally, both are `HashMap<String, OwnedFunction>`
#[derive(Default)]
pub struct ObjectFnMap(HashMap<String, OwnedFunction>);

impl Deref for ObjectFnMap {
    type Target = HashMap<String, OwnedFunction>;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl DerefMut for ObjectFnMap {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

/// Gets or creates a lua function that calls the specified proc on the object passed as its first argument.
///
/// Used in object indexing for object function calls of the form `object:proc()`
pub fn object_proccall_function(lua: &Lua, proc: String) -> LuaResult<OwnedFunction> {
    lua.app_data_mut::<ObjectFnMap>()
        .or_else(|| {
            lua.set_app_data::<ObjectFnMap>(ObjectFnMap::default());
            lua.app_data_mut()
        })
        .map(|mut map| {
            if let Some(func) = map.get(&proc).cloned() {
                Ok(func)
            } else {
                let proc_name = proc.clone();
                lua.create_function(move |_, (Value(this), args): (_, Variadic<Value>)| {
                    wrapped_object_call(&this, proc_name.clone(), args.to_vec())
                        .into_printed_external()
                })
                .map(Function::into_owned)
                .inspect(|func| {
                    map.insert(proc.clone(), func.clone());
                })
            }
        })
        .unwrap()
}
