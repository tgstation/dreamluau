use std::cell::RefCell;

use dreamluau_proc_macro::map_statics;
use meowtonin::{byond_fn, call_global, ByondResult, ToByond};

use crate::value::Value;

thread_local! {
    static GLOBAL_CALL_WRAPPER: RefCell<Option<String>> = const { RefCell::new(None) };
}

#[map_statics(mut GLOBAL_CALL_WRAPPER)]
#[byond_fn]
pub fn set_global_call_wrapper(new_wrapper: String) -> ByondResult<()> {
    if new_wrapper.is_empty() {
        global_call_wrapper.take();
    } else {
        global_call_wrapper.replace(if let Some(stripped) = new_wrapper.strip_prefix("/proc/") {
            stripped.to_string()
        } else {
            new_wrapper
        });
    }
    Ok(())
}

#[map_statics(GLOBAL_CALL_WRAPPER)]
fn get_global_call_wrapper() -> Option<String> {
    global_call_wrapper.clone()
}

pub fn wrapped_global_call<S: AsRef<str>, A: IntoIterator<Item = T> + ToByond, T: ToByond>(
    proc: S,
    args: A,
) -> ByondResult<Value> {
    if let Some(wrapper) = get_global_call_wrapper() {
        args.to_byond().and_then(|args_as_value| {
            call_global(wrapper, [proc.as_ref().to_byond().unwrap(), args_as_value])
        })
    } else {
        call_global(proc, args)
    }
}
