use std::cell::RefCell;

use dreamluau_proc_macro::map_statics;
use meowtonin::{byond_fn, call_global, ByondError, ByondResult, ByondValue, ToByond};

use crate::types::{type_name_for_obj, VARS_TYPES};

use super::error::WrapperError;

thread_local! {
    static VAR_SET_WRAPPER: RefCell<Option<String>> = const { RefCell::new(None) };
}

#[map_statics(mut VAR_SET_WRAPPER)]
#[byond_fn]
pub fn set_var_set_wrapper(new_wrapper: String) -> ByondResult<()> {
    if new_wrapper.is_empty() {
        var_set_wrapper.take();
    } else {
        var_set_wrapper.replace(if let Some(stripped) = new_wrapper.strip_prefix("/proc/") {
            stripped.to_string()
        } else {
            new_wrapper
        });
    }
    Ok(())
}

#[map_statics(VAR_SET_WRAPPER)]
fn get_set_var_wrapper() -> Option<String> {
    var_set_wrapper.clone()
}

pub fn wrapped_write_var(
    target: &mut ByondValue,
    var: String,
    value: &ByondValue,
) -> ByondResult<()> {
    if let Some(wrapper) = get_set_var_wrapper() {
        var.to_byond()
            .and_then(|ref var_as_value| call_global(wrapper, [target, var_as_value, value]))
    } else {
        target.write_var(var, value)
    }
}

pub fn wrapped_write_list_index<K: ToByond, V: ToByond>(
    target: &mut ByondValue,
    index: K,
    value: V,
) -> ByondResult<()> {
    if VARS_TYPES.contains(&target.get_type().0) && get_set_var_wrapper().is_some() {
        Err(ByondError::boxed(WrapperError::Forbidden {
            action: format!("direct modification of {} lists", type_name_for_obj(target)),
            wrapper: "var set".into(),
        }))
    } else {
        target.write_list_index(index, value)
    }
}
