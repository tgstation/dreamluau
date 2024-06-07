use std::cell::RefCell;

use dreamluau_proc_macro::map_statics;
use meowtonin::{byond_fn, call_global, ByondResult, ToByond};
use mlua::{
    prelude::{LuaError, LuaResult},
    Variadic,
};

use crate::{traits::AsPrintedExternalResult, value::Value};

use super::error::WrapperError;

thread_local! {
    static PRINT_WRAPPER: RefCell<Option<String>> = const { RefCell::new(None) };
}

#[map_statics(mut PRINT_WRAPPER)]
#[byond_fn]
pub fn set_print_wrapper(new_wrapper: String) -> ByondResult<()> {
    if new_wrapper.is_empty() {
        print_wrapper.take();
    } else {
        print_wrapper.replace(if let Some(stripped) = new_wrapper.strip_prefix("/proc/") {
            stripped.to_string()
        } else {
            new_wrapper
        });
    }
    Ok(())
}

#[map_statics(PRINT_WRAPPER)]
fn get_print_wrapper() -> Option<String> {
    print_wrapper.clone()
}

pub fn print(state_id: i32, args: Variadic<Value>) -> LuaResult<()> {
    get_print_wrapper()
        .ok_or_else(|| LuaError::external(WrapperError::NoWrapper("print")))
        .and_then(|wrapper| {
            call_global(
                wrapper,
                [
                    state_id.to_byond().map(Value).into_printed_external()?,
                    args.to_vec()
                        .to_byond()
                        .map(Value)
                        .into_printed_external()?,
                ],
            )
            .into_printed_external()
        })
}
