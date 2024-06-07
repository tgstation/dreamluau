use meowtonin::ByondValue;
use mlua::{
    prelude::{LuaResult, LuaValue},
    AnyUserData, Lua, Table,
};

use crate::value::{ByondObject, Value};

/// Get the userdata cache for this state.
///
/// The cache is a weak table of userdata indexed by DM reference IDs.
pub fn get_userdata_cache(lua: &Lua) -> LuaResult<Table<'_>> {
    lua.named_registry_value("userdata_cache")
        .and_then(|value| match value {
            LuaValue::Table(table) => Ok(table),
            _ => lua.create_table().and_then(|table| {
                lua.create_table()
                    .and_then(|metatable| {
                        metatable
                            .raw_set("__mode", "v")
                            .map(|()| table.set_metatable(Some(metatable)))
                    })
                    .and_then(|()| {
                        lua.set_named_registry_value("userdata_cache", table.clone())
                            .map(|()| table)
                    })
            }),
        })
}

/// Creates a userdata cache entry from the relevant bits of a DM value.
///
/// This is safe because there are only 40 bits of non-junk data in a DM value,
/// which fits in the 56-bit mantissa of an f64.
fn get_cache_id(value: &ByondValue) -> f64 {
    f64::from_bits(((value.0.type_ as u64) << 32) + unsafe { value.0.data.ref_ } as u64)
}

/// Get or create cached userdata for the passed in DM value.
///
/// Creating userdata this way increments the reference count of the underlying reference, preventing DM's garbage collector from collecting it.
/// The reference count is decremented when Lua garbage collects the userdata, allowing you to clear the hard reference at the DM level by
/// niling out all variables that object is assigned to.
///
/// If you absolutely must clear all references to a DM object while there still exists userdata for it, call `drop_cached_userdata`
pub fn get_or_create_cached_userdata(Value(value): Value, lua: &Lua) -> LuaResult<AnyUserData<'_>> {
    get_userdata_cache(lua).and_then(|cache| {
        let id = LuaValue::Number(get_cache_id(&value));
        cache
            .raw_get(id.clone())
            .and_then(|cache_value| match cache_value {
                LuaValue::UserData(ud) => Ok(ud),
                _ => lua
                    .create_userdata(ByondObject::new(value))
                    .and_then(|ud| cache.raw_set(id, ud.clone()).map(|()| ud)),
            })
    })
}

pub fn drop_cached_userdata(Value(value): &Value, lua: &Lua) -> LuaResult<()> {
    get_userdata_cache(lua).and_then(|cache| {
        let id = get_cache_id(value);
        cache.raw_get::<_, Option<AnyUserData>>(id).and_then(|opt| {
            if let Some(ud) = opt {
                cache.raw_remove(id)?;
                ud.take::<ByondObject>()?;
            }
            Ok(())
        })
    })
}
