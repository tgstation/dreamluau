use mlua::prelude::{LuaError, LuaResult};
pub trait AsPrintedExternal {
    fn into_printed_external(self) -> LuaError;
}

impl<T> AsPrintedExternal for T
where
    T: ToString,
{
    fn into_printed_external(self) -> LuaError {
        LuaError::external(self.to_string())
    }
}

pub trait AsPrintedExternalResult<T> {
    fn into_printed_external(self) -> LuaResult<T>;
}

impl<T, E> AsPrintedExternalResult<T> for Result<T, E>
where
    E: AsPrintedExternal,
{
    fn into_printed_external(self) -> LuaResult<T> {
        self.map_err(|e| e.into_printed_external())
    }
}
