use std::cell::RefCell;
use std::error::Error;
use std::ptr;
use std::rc::Rc;

use meowtonin::{byond_fn, ByondError, ByondResult, ByondValue, ToByond};
use mlua::{Compiler, Lua, Table};

use dreamluau_proc_macro::map_statics;
use exec_limit::limiting_interrupt;

use crate::cache::global_proc::GlobalFnMap;
use crate::cache::object_proc::ObjectFnMap;
use crate::cache::userdata::drop_cached_userdata;
use crate::value::{safe_convert_from_table, Value};

use self::library::{GlobalModule, LuaModule, PackageModule};
use self::threads::{
    get_yielded_thread, nuke_main_chunks, remove_sleeping_thread, ThreadList, Threads,
};
use self::util::entrypoint::{get_entrypoint, remove_main_chunk};
use self::util::prepare_registry_functions;
pub use exec_limit::{clear_execution_limit, set_execution_limit_millis, set_execution_limit_secs};
pub use usr::set_usr;
pub use util::traceback::get_traceback;

mod exec_limit;
mod library;
mod run;
mod sleep;
mod threads;
mod usr;
mod util;

thread_local! {
    pub static STATES: RefCell<Vec<Option<Rc<Lua>>>> = const { RefCell::new(vec![]) };
}

#[map_statics(mut STATES)]
#[byond_fn]
pub fn new_state(isolate: Option<bool>) -> ByondResult<usize> {
    let lua: Lua = Lua::new();
    lua.set_named_registry_value("isolated", isolate.unwrap_or(false))
        .map_err(ByondError::boxed)?;
    let new_state_index = states.iter().position(Option::is_none).unwrap_or_else(|| {
        states.push(None);
        states.len() - 1
    });
    lua.set_interrupt(limiting_interrupt);
    GlobalModule(new_state_index as i32)
        .populate_table(&lua.globals(), &lua)
        .and_then(|()| lua.globals().raw_get::<_, Table>("package"))
        .and_then(|package| PackageModule.populate_table(&package, &lua))
        .and_then(|()| prepare_registry_functions(&lua))
        .and_then(|()| lua.sandbox(true))
        .map_err(ByondError::boxed)?;
    lua.set_compiler(
        Compiler::new().set_mutable_globals(["dm", "exec"].map(String::from).to_vec()),
    );
    states[new_state_index].replace(Rc::new(lua));
    Ok(new_state_index)
}

#[map_statics(STATES)]
fn get_state(index: usize) -> ByondResult<Rc<Lua>> {
    states
        .get(index)
        .and_then(Option::as_ref)
        .cloned()
        .ok_or(ByondError::Boxed(Box::<dyn Error + Send + Sync>::from(
            format!("No state at index {index}"),
        )))
}

#[byond_fn]
pub fn is_isolated(index: usize) -> ByondResult<bool> {
    get_state(index).and_then(|lua| {
        lua.named_registry_value("isolated")
            .map_err(ByondError::boxed)
    })
}

#[byond_fn]
pub fn load(index: usize, code: String, name: Option<String>) -> ByondResult<ByondValue> {
    get_state(index).and_then(|lua| run::load(lua.as_ref(), code, name))
}

#[byond_fn]
pub fn awaken(index: usize) -> ByondResult<ByondValue> {
    get_state(index).and_then(|lua| run::awaken(lua.as_ref()))
}

#[byond_fn]
pub fn resume(
    state_index: usize,
    thread_index: usize,
    args: Vec<Value>,
) -> ByondResult<ByondValue> {
    get_state(state_index).and_then(|lua| run::resume(lua.as_ref(), thread_index, args))
}

#[byond_fn]
pub fn call_function(index: usize, path: Vec<Value>, args: Vec<Value>) -> ByondResult<ByondValue> {
    get_state(index).and_then(|lua| run::call(lua.as_ref(), path, args))
}

#[byond_fn]
pub fn get_globals(index: usize) -> ByondResult<Value> {
    get_state(index)
        .and_then(|lua| {
            safe_convert_from_table(lua.as_ref(), lua.globals()).map_err(ByondError::boxed)
        })
        .and_then(|(values, variants)| {
            vec![
                ("values", values),
                ("variants", variants.to_byond().map(Value)?),
            ]
            .to_byond()
            .map(Value)
        })
}

#[byond_fn]
pub fn list_threads(index: usize) -> ByondResult<ThreadList> {
    get_state(index).and_then(|lua| threads::list_threads(lua.as_ref()))
}

#[byond_fn]
pub fn collect_garbage(index: usize) -> ByondResult<()> {
    get_state(index).and_then(|lua| {
        lua.gc_collect().map_err(ByondError::boxed)?;
        lua.gc_collect().map_err(ByondError::boxed)
    })
}

#[byond_fn]
pub fn kill_yielded_thread(state_index: usize, thread_index: usize) -> ByondResult<()> {
    get_state(state_index).and_then(|lua| {
        get_yielded_thread(lua.as_ref(), thread_index)
            .map(|thread| {
                remove_main_chunk(
                    &get_entrypoint(lua.as_ref(), &thread.thread).unwrap_or(ptr::null()),
                )
            })
            .map_err(ByondError::boxed)
    })
}

#[byond_fn]
pub fn kill_sleeping_thread(state_index: usize, thread_index: usize) -> ByondResult<()> {
    get_state(state_index).and_then(|lua| {
        remove_sleeping_thread(lua.as_ref(), thread_index)
            .map(|thread| {
                remove_main_chunk(
                    &get_entrypoint(lua.as_ref(), &thread.thread).unwrap_or(ptr::null()),
                )
            })
            .map_err(ByondError::boxed)
    })
}

/// Removes all the app data from the passed in state.
///
/// The app data associated with a state contains hard references to that state.
/// Since the container for app data is part of a struct stored in the registry,
/// a hard reference cycle is created, preventing the state from being dropped.
/// Thus, the app data containing these hard references must be removed.
fn nuke_app_data(lua: &Lua) {
    lua.remove_app_data::<Threads>();
    lua.remove_app_data::<ObjectFnMap>();
    lua.remove_app_data::<GlobalFnMap>();
}

#[map_statics(mut STATES)]
#[byond_fn]
pub fn kill_state(index: usize) -> ByondResult<()> {
    states
        .get_mut(index)
        .ok_or(ByondError::Boxed(Box::<dyn Error + Send + Sync>::from(
            format!("No state at index {index}"),
        )))
        .and_then(|opt| {
            if opt.as_ref().is_some_and(|rc| Rc::strong_count(rc) == 1) {
                let mut rc = opt.take().unwrap();
                let lua = Rc::get_mut(&mut rc).unwrap();
                nuke_main_chunks(lua);
                nuke_app_data(lua);
                Ok(())
            } else {
                Err(ByondError::Boxed(Box::<dyn Error + Send + Sync>::from(
                    format!("State at index {index} is still in use"),
                )))
            }
        })
}

#[map_statics(STATES)]
#[byond_fn]
pub fn clear_ref_userdata(value: ByondValue) -> ByondResult<()> {
    states
        .iter()
        .filter_map(|opt| opt.as_deref())
        .try_for_each(|lua| drop_cached_userdata(&Value(value.clone()), lua))
        .map_err(ByondError::boxed)
}
