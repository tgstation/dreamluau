use std::cell::RefCell;

use dreamluau_proc_macro::map_statics;
use meowtonin::{byond_fn, call_global, ByondResult, ByondValue, ToByond};

use crate::value::Value;

thread_local! {
    static NEW_WRAPPER: RefCell<Option<String>> = const { RefCell::new(None) };
}

#[map_statics(mut NEW_WRAPPER)]
#[byond_fn]
pub fn set_new_wrapper(new_new_wrapper: String) -> ByondResult<()> {
    if new_new_wrapper.is_empty() {
        new_wrapper.take();
    } else {
        new_wrapper.replace(
            if let Some(stripped) = new_new_wrapper.strip_prefix("/proc/") {
                stripped.to_string()
            } else {
                new_new_wrapper
            },
        );
    }
    Ok(())
}

#[map_statics(NEW_WRAPPER)]
fn get_new_wrapper() -> Option<String> {
    new_wrapper.clone()
}

pub fn wrapped_new<S: Into<String>, A: IntoIterator<Item = T> + ToByond, T: ToByond>(
    typepath: S,
    args: A,
) -> ByondResult<Value> {
    if let Some(wrapper) = get_new_wrapper() {
        args.to_byond().and_then(|args_as_value| {
            call_global(
                wrapper,
                [typepath.into().to_byond().unwrap(), args_as_value],
            )
        })
    } else {
        ByondValue::new(
            typepath,
            args.into_iter()
                .map(|t| t.to_byond())
                .collect::<ByondResult<Vec<ByondValue>>>()?
                .as_slice(),
        )
        .map(Value)
    }
}
