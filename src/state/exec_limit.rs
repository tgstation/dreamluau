use std::{
    cell::RefCell,
    time::{Duration, Instant, TryFromFloatSecsError},
};

use dreamluau_proc_macro::map_statics;
use meowtonin::byond_fn;
use mlua::{
    prelude::{LuaError, LuaResult},
    Lua, VmState,
};

thread_local! {
    static EXECUTION_LIMIT: RefCell<Option<Duration>> = const { RefCell::new(Some(Duration::from_millis(100))) };
    static EXECUTION_START: RefCell<Option<Instant>> = const { RefCell::new(None) };
    pub static CALL_DEPTH: RefCell<usize> = const { RefCell::new(0) };
    static PRIVILEGED_EXECUTION: RefCell<bool> = const { RefCell::new(false) };
}

/// Sets the execution limit in milliseconds
#[map_statics(mut EXECUTION_LIMIT)]
#[byond_fn]
pub fn set_execution_limit_millis(new_limit: u32) {
    execution_limit.replace(Duration::from_millis(new_limit as u64));
}

/// Sets the execution limit in seconds
#[map_statics(mut EXECUTION_LIMIT)]
#[byond_fn]
pub fn set_execution_limit_secs(new_limit: f32) -> Result<(), TryFromFloatSecsError> {
    Duration::try_from_secs_f32(new_limit).map(|d| {
        execution_limit.replace(d);
    })
}

#[map_statics(mut EXECUTION_LIMIT)]
#[byond_fn]
pub fn clear_execution_limit() {
    execution_limit.take();
}

#[map_statics(EXECUTION_LIMIT)]
pub fn get_execution_limit() -> Option<Duration> {
    *execution_limit
}

#[map_statics(mut EXECUTION_START, mut CALL_DEPTH)]
pub fn increment_call_depth() {
    if *call_depth == 0 {
        execution_start.replace(Instant::now());
    }
    *call_depth += 1
}

#[map_statics(mut EXECUTION_START, mut CALL_DEPTH)]
pub fn decrement_call_depth() {
    *call_depth -= 1;
    if *call_depth == 0 {
        execution_start.take();
    }
}

#[map_statics(EXECUTION_START)]
pub fn get_execution_time() -> Option<u128> {
    execution_start.map(|start| start.elapsed().as_millis())
}

#[map_statics(mut PRIVILEGED_EXECUTION)]
pub fn set_privileged_execution(privileged: bool) {
    *privileged_execution = privileged;
}

#[map_statics(EXECUTION_START, EXECUTION_LIMIT, PRIVILEGED_EXECUTION)]
pub fn limiting_interrupt(_: &Lua) -> LuaResult<VmState> {
    match (execution_limit, execution_start, privileged_execution) {
        (_, _, true) => Ok(VmState::Continue),
        (Some(limit), Some(start), _) => (start.elapsed() <= *limit)
            .then_some(VmState::Continue)
            .ok_or_else(|| {
                LuaError::external(
                    "execution limit reached - call sleep or coroutine.yield before this point",
                )
            }),
        (_, _, _) => Ok(VmState::Continue),
    }
}
