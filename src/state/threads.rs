use mlua::OwnedThread;

#[derive(Clone)]
pub struct NamedThread {
    pub(crate) name: String,
    pub(crate) thread: OwnedThread,
}

use std::collections::VecDeque;

use meowtonin::{ByondResult, ByondValue, ToByond};
use mlua::{
    prelude::{LuaError, LuaResult, LuaValue},
    AppDataRef, AppDataRefMut, Lua,
};

use super::{
    sleep::take_sleep_flag,
    util::entrypoint::{get_entrypoint, remove_main_chunk},
};
#[derive(Default)]
pub struct Threads {
    pub yields: Vec<Option<NamedThread>>,
    pub sleeps: VecDeque<NamedThread>,
}

fn get_thread_storage(lua: &Lua) -> AppDataRef<Threads> {
    lua.app_data_ref::<Threads>()
        .or_else(|| {
            lua.set_app_data::<Threads>(Threads::default());
            lua.app_data_ref()
        })
        .unwrap()
}

fn get_thread_storage_mut(lua: &Lua) -> AppDataRefMut<Threads> {
    lua.app_data_mut::<Threads>()
        .or_else(|| {
            lua.set_app_data::<Threads>(Threads::default());
            lua.app_data_mut()
        })
        .unwrap()
}

pub fn push_yielded_thread(lua: &Lua, thread: NamedThread) -> LuaResult<Option<usize>> {
    let mut storage = get_thread_storage_mut(lua);
    if take_sleep_flag() {
        storage.sleeps.push_back(thread);
        Ok(None)
    } else if let Some(index) = storage.yields.iter().position(Option::is_none) {
        storage.yields[index].replace(thread);
        Ok(Some(index))
    } else {
        storage.yields.push(Some(thread));
        Ok(Some(storage.yields.len() - 1))
    }
}

pub fn next_yield_index(lua: &Lua) -> LuaResult<LuaValue> {
    let storage = get_thread_storage(lua);
    let yields = &storage.yields;
    yields
        .iter()
        .position(Option::is_none)
        .unwrap_or(yields.len())
        .try_into()
        .map(LuaValue::Integer)
        .map_err(LuaError::external)
}

pub fn get_yielded_thread(lua: &Lua, index: usize) -> LuaResult<NamedThread> {
    let mut storage = get_thread_storage_mut(lua);
    let yields = &mut storage.yields;
    if (0..yields.len()).contains(&index) {
        let opt = &mut yields[index];
        opt.take().ok_or_else(|| {
            LuaError::external(format!("No yielded thread at index {index}").as_str())
        })
    } else {
        Err(LuaError::external("Index out of bounds"))
    }
}

pub fn pop_front_sleeping_thread(lua: &Lua) -> LuaResult<NamedThread> {
    let mut storage = get_thread_storage_mut(lua);
    storage
        .sleeps
        .pop_front()
        .ok_or_else(|| LuaError::external("Sleep queue is empty"))
}

pub fn remove_sleeping_thread(lua: &Lua, index: usize) -> LuaResult<NamedThread> {
    let mut storage = get_thread_storage_mut(lua);
    storage
        .sleeps
        .remove(index)
        .ok_or_else(|| LuaError::external("Index out of bounds".to_string().as_str()))
}

pub type ThreadList = Vec<(&'static str, Vec<Vec<(&'static str, ByondValue)>>)>;

pub fn list_threads(lua: &Lua) -> ByondResult<ThreadList> {
    let storage = get_thread_storage(lua);
    Ok(vec![
        (
            "yields",
            storage
                .yields
                .iter()
                .map(|opt| opt.as_ref().map(|NamedThread { name, thread: _ }| name))
                .enumerate()
                .filter_map(|(i, opt)| {
                    opt.map(|name| {
                        Ok(vec![
                            ("index", i.to_byond()?),
                            ("name", name.to_byond().unwrap()),
                        ])
                    })
                })
                .collect::<ByondResult<_>>()?,
        ),
        (
            "sleeps",
            storage
                .sleeps
                .iter()
                .map(|NamedThread { name, thread: _ }| name)
                .enumerate()
                .map(|(i, name)| {
                    Ok(vec![
                        ("index", i.to_byond()?),
                        ("name", name.to_byond().unwrap()),
                    ])
                })
                .collect::<ByondResult<_>>()?,
        ),
    ])
}

pub fn nuke_main_chunks(lua: &Lua) {
    let storage = get_thread_storage(lua);
    storage
        .sleeps
        .iter()
        .chain(storage.yields.iter().filter_map(|o| o.as_ref()))
        .map(|thread| get_entrypoint(lua, &thread.thread))
        .filter_map(Result::ok)
        .for_each(|f| remove_main_chunk(&f));
}
