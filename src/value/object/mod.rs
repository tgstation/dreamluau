use std::ffi::CString;

use self::indexing::validate_index;

use super::Value;

use meowtonin::{call_global, ByondError, ByondValue, FromByond};

use crate::{
    cache::object_proc::object_proccall_function,
    helpers::WORLD,
    traits::AsPrintedExternal,
    types::PROC_DEFINABLE_TYPES,
    wrappers::{
        wrapped_read_list_index, wrapped_read_var, wrapped_write_list_index, wrapped_write_var,
    },
};

use mlua::{Function, IntoLua, IntoLuaMulti, MetaMethod, MultiValue, UserData, UserDataMethods};

mod indexing;

use crate::traits::AsPrintedExternalResult;

#[derive(Clone)]
pub struct ByondObject(pub(crate) ByondValue);

impl ByondObject {
    pub fn new(value: ByondValue) -> Self {
        value.inc_ref();
        Self(value)
    }
}

impl UserData for ByondObject {
    fn add_methods<'lua, M: UserDataMethods<'lua, Self>>(methods: &mut M) {
        methods.add_meta_function(MetaMethod::Eq, |_, (Value(left), Value(right))| {
            Ok(left == right)
        });
        methods.add_meta_function(MetaMethod::Len, |_, Value(this)| {
            if this.is_list() {
                Ok(this.length::<u32>().into_printed_external())
            } else {
                Err(ByondError::NotAList.into_printed_external())
            }
        });
        methods.add_meta_function(MetaMethod::Iter, |_, Value(this)| {
            if this.is_list() {
                Ok((
                    Function::wrap(|lua, (Value(this), last_index): (Value, u32)| {
                        let index = last_index + 1;
                        if index > this.length::<u32>().unwrap() {
                            return Ok(MultiValue::new());
                        }
                        this.read_list_index::<_, Value>(&index)
                            .into_printed_external()
                            .and_then(|v| (index, v).into_lua_multi(lua))
                    }),
                    Value(this),
                    0,
                ))
            } else {
                Err(ByondError::NotAList.into_printed_external())
            }
        });
        methods.add_meta_function(MetaMethod::ToString, |_, Value(this)| {
            CString::from_byond(&this).into_printed_external()
        });
        methods.add_meta_function(
            MetaMethod::NewIndex,
            |_, (Value(ref mut this), Value(ref index), Value(ref value))| {
                validate_index(this, index).and_then(|()| {
                    if this.is_list() {
                        wrapped_write_list_index(this, index, value).into_printed_external()
                    } else {
                        wrapped_write_var(this, index.get_string().unwrap(), value)
                            .into_printed_external()
                    }
                })
            },
        );
        methods.add_meta_function(
            MetaMethod::Index,
            |lua, (Value(ref this), Value(ref index))| {
                validate_index(this, index)
                    .and_then(|()| {
                        if this.is_list() {
                            wrapped_read_list_index(this, index)
                                .into_printed_external()
                                .and_then(|v| v.into_lua(lua))
                        } else if (PROC_DEFINABLE_TYPES.contains(&this.get_type().0)
                            || *this == WORLD)
                            && call_global::<_, _, _, bool>("_hascall", [this, index])
                                .into_printed_external()?
                        {
                            object_proccall_function(lua, index.get_string().unwrap())
                                .and_then(|v| v.into_lua(lua))
                        } else {
                            wrapped_read_var(this, index.get_string().unwrap())
                                .into_printed_external()
                                .and_then(|v| v.into_lua(lua))
                        }
                    })
                    .and_then(|b| b.into_lua(lua))
            },
        )
    }
}

impl Drop for ByondObject {
    fn drop(&mut self) {
        let value = &self.0;
        value.dec_ref();
    }
}
