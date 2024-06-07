use meowtonin::ByondValue;
use mlua::prelude::{LuaError, LuaResult};

use crate::{
    traits::AsPrintedExternalResult,
    types::{
        can_index_at_all, can_index_by_anything, can_index_by_number, can_index_by_string,
        is_valid_proc_index_for_value, is_valid_var_index_for_value, type_name_for_obj,
    },
};

/// Checks if `object` can be indexed into with the given `index`.
///
/// Accounts for types that can only be indexed by certain kinds of values
/// (e.g. datums and datum-like types can only be indexed by string, non-standard lists can only be indexed by string or number)
pub fn validate_index(object: &ByondValue, index: &ByondValue) -> LuaResult<()> {
    if !can_index_at_all(object) {
        Err(LuaError::external(format!(
            "Cannot index objects of type \"{}\"",
            type_name_for_obj(object)
        )))
    } else if can_index_by_anything(object) {
        Ok(())
    } else if can_index_by_string(object) {
        if index.is_string() {
            let index_string = index.get_string_bytes().into_printed_external()?;
            if is_valid_var_index_for_value(index_string.as_slice(), object)
                || std::str::from_utf8(index_string.as_slice())
                    .is_ok_and(|index_str| is_valid_proc_index_for_value(index_str, object))
            {
                Ok(())
            } else if let Ok(index_string) = String::from_utf8(index_string) {
                Err(LuaError::external(format!(
                    "\"{index_string}\" is not a valid index for objects of type \"{}\"",
                    type_name_for_obj(object)
                )))
            } else {
                Err(LuaError::external(format!(
                    "Invalid UTF-8 strings cannot be indices for objects of type \"{}\"",
                    type_name_for_obj(object)
                )))
            }
        } else if index.is_number() {
            if can_index_by_number(object) {
                Ok(())
            } else {
                Err(LuaError::external(format!(
                    "Objects of type \"{}\" can only be indexed by string, got \"{}\"",
                    type_name_for_obj(object),
                    type_name_for_obj(index)
                )))
            }
        } else {
            Err(LuaError::external(format!(
                "Objects of type \"{}\" can only be indexed by string or number, got \"{}\"",
                type_name_for_obj(object),
                type_name_for_obj(index)
            )))
        }
    } else if can_index_by_number(object) {
        if index.is_number() {
            Ok(())
        } else {
            Err(LuaError::external(format!(
                "Objects of type \"{}\" can only be indexed by number, got \"{}\"",
                type_name_for_obj(object),
                type_name_for_obj(index)
            )))
        }
    } else {
        Err(LuaError::external(format!(
            "Attempted indexing {} with {} (all possible cases should have been accounted for)",
            type_name_for_obj(object),
            type_name_for_obj(index)
        )))
    }
}
