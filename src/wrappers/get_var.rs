use std::cell::RefCell;

use dreamluau_proc_macro::map_statics;
use meowtonin::{byond_fn, call_global, ByondError, ByondResult, ByondValue, ToByond};

use crate::{
    types::{type_name_for_obj, VARS_TYPES},
    value::Value,
};

use super::error::WrapperError;

thread_local! {
    static VAR_GET_WRAPPER: RefCell<Option<String>> = const { RefCell::new(None) };
}

#[map_statics(mut VAR_GET_WRAPPER)]
#[byond_fn]
pub fn set_var_get_wrapper(new_wrapper: String) -> ByondResult<()> {
    if new_wrapper.is_empty() {
        var_get_wrapper.take();
    } else {
        var_get_wrapper.replace(if let Some(stripped) = new_wrapper.strip_prefix("/proc/") {
            stripped.to_string()
        } else {
            new_wrapper
        });
    }
    Ok(())
}

#[map_statics(VAR_GET_WRAPPER)]
fn get_get_var_wrapper() -> Option<String> {
    var_get_wrapper.clone()
}

pub fn wrapped_read_var(target: &ByondValue, var: String) -> ByondResult<Value> {
    if let Some(wrapper) = get_get_var_wrapper() {
        var.to_byond()
            .and_then(|var_as_value| call_global(wrapper, [target, &var_as_value]))
    } else {
        target.read_var(var)
    }
}

pub fn wrapped_read_list_index<K: ToByond>(target: &ByondValue, index: K) -> ByondResult<Value> {
    if VARS_TYPES.contains(&target.get_type().0)
        && get_get_var_wrapper().is_some()
        && index.to_byond().is_ok_and(|v| !v.is_number())
    {
        Err(ByondError::boxed(WrapperError::Forbidden {
            action: format!(
                "direct reading of {} assoc values",
                type_name_for_obj(target)
            ),
            wrapper: "var get".into(),
        }))
    } else {
        target.read_list_index(&index)
    }
}
