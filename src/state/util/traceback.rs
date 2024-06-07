use dreamluau_proc_macro::map_statics;
use meowtonin::{byond_fn, ByondError, ByondResult, ByondValue, FromByond};
use mlua::{
    ffi::{
        luaL_Strbuf, luaL_addchar, luaL_addlstring, luaL_buffinit, luaL_pushresult, lua_Debug,
        lua_getargument, lua_getinfo, lua_mainthread, lua_topointer, lua_tothread, lua_xmove,
    },
    lua_State,
    prelude::{LuaResult, LuaValue},
    Function, Lua, OwnedFunction, OwnedThread,
};
use std::{
    cell::RefCell,
    ffi::{CStr, CString},
    mem,
};

use crate::value::ByondObject;

use super::{super::exec_limit::set_privileged_execution, entrypoint::is_main_chunk};

thread_local! {
    static TRACEBACK_STACK: RefCell<Vec<OwnedFunction>> = const {RefCell::new(vec![])};
}

/// Produces a traceback of the passed in thread, formatted the same way BYOND formats runtime stack traces.
///
/// Ignores C functions and lua functions defined by the `mlua` dependency, because our entrypoints and userdata hooks should be considered transparent.
pub unsafe extern "C-unwind" fn dm_traceback(lua: *mut lua_State) -> i32 {
    let lua1 = lua_tothread(lua, -1);
    let main = lua_mainthread(lua);
    let main_struct = Lua::init_from_ptr(main);

    let mut buf: luaL_Strbuf = mem::zeroed();
    let mut ar: lua_Debug = mem::zeroed();
    let what = CString::new("nfs").unwrap();
    luaL_buffinit(lua, &mut buf);
    let mut level = 1;
    let mut add_newline = false;
    while lua_getinfo(lua1, level, what.as_ptr(), &mut ar) == 1 {
        let fptr = lua_topointer(lua1, -1);
        let fsrc = (!ar.source.is_null()).then(|| CStr::from_ptr(ar.source));
        if CStr::from_ptr(ar.what).to_string_lossy() == "C"
            || fsrc.is_some_and(|src| src.to_string_lossy().starts_with("__mlua"))
        {
            level += 1;
            continue;
        }
        if add_newline {
            luaL_addchar(&mut buf, b'\n' as i8);
        }
        let fname: Vec<u8> = if is_main_chunk(&fptr) {
            match fsrc {
                Some(cstr) => {
                    let bytes = cstr.to_bytes();
                    b"main chunk \""
                        .iter()
                        .copied()
                        .chain(bytes.escape_ascii())
                        .chain(b"\"".to_owned())
                        .collect()
                }
                None => b"anonymous chunk".to_vec(),
            }
        } else if ar.name.is_null() {
            Vec::from(format!("anonymous function ({fptr:p})"))
        } else {
            CStr::from_ptr(ar.name)
                .to_bytes()
                .iter()
                .copied()
                .map(|b| match b {
                    b'_' => b' ',
                    _ => b,
                })
                .collect()
        };
        luaL_addlstring(&mut buf, fname.as_ptr() as *const i8, fname.len());
        luaL_addchar(&mut buf, b'(' as i8);
        let mut arg = 1;
        while lua_getargument(lua1, level, arg) == 1 {
            if arg > 1 {
                luaL_addlstring(&mut buf, ", ".as_ptr() as *const i8, 2);
            }
            lua_xmove(lua1, main, 1);
            let arg_string = match main_struct.pop_value() {
                LuaValue::Nil => b"null".to_vec(),
                LuaValue::UserData(u) if u.is::<ByondObject>() => {
                    let obj = &u.borrow::<ByondObject>().unwrap().0;
                    let mut ostring = CString::from_byond(obj)
                        .map(CString::into_bytes)
                        .unwrap_or_else(|_| b"???".to_vec());
                    if ostring.len() >= 30 {
                        ostring.truncate(30);
                        ostring.extend(b"...");
                    };
                    if obj.is_list() {
                        ostring.extend(b" (/list)");
                    } else if let Ok(typepath) = obj.typepath() {
                        ostring.extend(format!(" ({typepath})").as_bytes());
                    }
                    ostring
                }
                LuaValue::String(s) => {
                    let mut bytes: Vec<u8> = s.as_bytes().into();
                    if bytes.len() >= 30 {
                        bytes.truncate(30);
                        bytes.extend(b"...");
                    };
                    "\"".bytes()
                        .chain(bytes.escape_ascii())
                        .chain("\"".bytes())
                        .collect()
                }
                LuaValue::Number(n) => n.to_string().as_bytes().to_vec(),
                LuaValue::Integer(i) => i.to_string().as_bytes().to_vec(),
                LuaValue::Function(f) => match f.info().name {
                    Some(n) => format!("function {} ({:p})", n, f.to_pointer()),
                    None => format!("anonymous function ({:p})", f.to_pointer()),
                }
                .as_bytes()
                .to_vec(),
                anything_else => Vec::from(format!(
                    "{:p} ({})",
                    anything_else.to_pointer(),
                    anything_else.type_name()
                )),
            };
            luaL_addlstring(&mut buf, arg_string.as_ptr() as *const i8, arg_string.len());
            arg += 1;
        }
        luaL_addchar(&mut buf, b')' as i8);
        level += 1;
        add_newline = true;
    }
    luaL_pushresult(&mut buf);
    1
}

#[map_statics(mut TRACEBACK_STACK)]
pub fn push_traceback_func(lua: &Lua, thread: &OwnedThread) -> LuaResult<()> {
    traceback_stack.push(
        lua.named_registry_value::<Function>("dm_traceback")?
            .bind(thread)?
            .into_owned(),
    );
    Ok(())
}

#[map_statics(mut TRACEBACK_STACK)]
pub fn pop_traceback_func() {
    traceback_stack.pop();
}

#[map_statics(TRACEBACK_STACK)]
#[byond_fn]
pub fn get_traceback(index: usize) -> ByondResult<ByondValue> {
    if index == 0 || index > traceback_stack.len() {
        Ok(ByondValue::null())
    } else {
        set_privileged_execution(true); // We don't want the traceback function to fall afoul of the execution limit.
        let ret = traceback_stack[traceback_stack.len() - index]
            .call::<_, String>(())
            .map(ByondValue::new_string)
            .map_err(ByondError::boxed);
        set_privileged_execution(false);
        ret
    }
}
