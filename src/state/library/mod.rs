use std::collections::HashMap;

use mlua::{
    prelude::{LuaResult, LuaValue},
    Function, IntoLua, Lua, MetaMethod, Table,
};

mod dm;
mod exec;
mod global;
mod global_procs;
mod list;
mod package;
mod pointer;
pub use global::GlobalModule;
pub use package::PackageModule;

pub fn fill_table_from<'lua, K, V, I>(table: &Table<'lua>, i: I) -> LuaResult<()>
where
    K: IntoLua<'lua>,
    V: IntoLua<'lua>,
    I: IntoIterator<Item = (K, V)>,
{
    i.into_iter()
        .try_for_each(|(key, value)| table.raw_set(key, value))
}

pub fn create_metafields<'lua, I, F>(
    lua: &'lua Lua,
    metafields: I,
    fallback_getter: Option<LuaValue<'lua>>,
    fallback_setter: Option<LuaValue<'lua>>,
) -> LuaResult<HashMap<&'static str, LuaValue<'lua>>>
where
    I: IntoIterator<Item = (String, F)>,
    F: Fn(&'lua Lua) -> LuaResult<LuaValue<'lua>> + 'static,
{
    let metafields: HashMap<String, F> = HashMap::from_iter(metafields);
    let reserved_names = metafields
        .iter()
        .unzip::<&String, &F, Vec<&String>, Vec<&F>>()
        .0
        .into_iter()
        .cloned()
        .collect::<Vec<String>>();
    let fallback_getter = match fallback_getter {
        Some(v) => lua
            .create_function(|_, (fallback, this, index): (LuaValue, Table, LuaValue)| {
                match fallback {
                    LuaValue::Function(f) => f.call((this, index)),
                    LuaValue::Table(t) => t.get(index),
                    otherwise => Ok(otherwise),
                }
            })
            .and_then(|f| f.bind(v)),
        None => lua.create_function(|_, (this, index): (Table, LuaValue)| {
            this.raw_get::<_, LuaValue>(index)
        }),
    }?;
    let fallback_setter = match fallback_setter {
        Some(v) => lua
            .create_function(
                |_, (fallback, this, index, value): (LuaValue, Table, LuaValue, LuaValue)| {
                    match fallback {
                        LuaValue::Function(f) => f.call((this, index, value)),
                        LuaValue::Table(t) => t.set(index, value),
                        _ => Ok(()),
                    }
                },
            )
            .and_then(|f| f.bind(v)),
        None => lua.create_function(|_, (this, index, value): (Table, LuaValue, LuaValue)| {
            this.raw_set(index, value)
        }),
    }?;
    Ok([
        (
            MetaMethod::Index.name(),
            lua.create_function(
                move |lua, (fallback, this, index): (Function, Table, LuaValue)| {
                    if let LuaValue::String(index) = index.clone() {
                        if let Ok(i) = index.to_str() {
                            if let Some(f) = metafields
                                .iter()
                                .find_map(|(k, f)| (i == k.as_str()).then_some(f))
                            {
                                return f(lua);
                            }
                        }
                    }
                    fallback.call((this, index))
                },
            )
            .and_then(|f| f.bind(fallback_getter))
            .map(LuaValue::Function)?,
        ),
        (
            MetaMethod::NewIndex.name(),
            lua.create_function(
                move |_, (fallback, this, index, value): (Function, Table, LuaValue, LuaValue)| {
                    if let LuaValue::String(index) = index.clone() {
                        if let Ok(i) = index.to_str() {
                            if reserved_names.contains(&String::from(i)) {
                                return Ok(());
                            }
                        }
                    }
                    fallback.call::<_, ()>((this, index, value))
                },
            )
            .and_then(|f| f.bind(fallback_setter))
            .map(LuaValue::Function)?,
        ),
    ]
    .into())
}

type MetafieldFn = dyn for<'lua> Fn(&'lua Lua) -> LuaResult<LuaValue<'lua>>;
type MetafieldItems = Vec<(String, Box<MetafieldFn>)>;

pub trait LuaModule {
    fn create_items<'lua>(&self, _: &'lua Lua) -> LuaResult<Vec<(&str, LuaValue<'lua>)>> {
        Ok(Vec::default())
    }

    fn is_readonly(&self) -> bool {
        true
    }

    fn create_metafield_items(&self) -> Option<MetafieldItems> {
        None
    }

    fn create_metamethods<'lua>(
        &self,
        _: &'lua Lua,
    ) -> LuaResult<HashMap<&'static str, LuaValue<'lua>>> {
        Ok(HashMap::new())
    }

    fn populate_table<'lua>(&self, table: &Table<'lua>, lua: &'lua Lua) -> LuaResult<()> {
        fill_table_from(table, self.create_items(lua)?)?;
        let mut metamethods = self.create_metamethods(lua)?;
        if let Some(metafield_items) = self.create_metafield_items() {
            let metafield_metamethods = create_metafields(
                lua,
                metafield_items,
                metamethods.remove(&MetaMethod::Index.name()),
                metamethods.remove(&MetaMethod::NewIndex.name()),
            )?;
            metamethods.extend(metafield_metamethods)
        }
        let metatable = lua.create_table()?;
        table.set_metatable(Some(metatable.clone()));
        table.set_readonly(self.is_readonly());
        fill_table_from(&metatable, metamethods)?;
        Ok(())
    }
}

impl<'lua> IntoLua<'lua> for &dyn LuaModule {
    fn into_lua(self, lua: &'lua Lua) -> LuaResult<LuaValue<'lua>> {
        lua.create_table().and_then(|table| {
            self.populate_table(&table, lua)?;
            Ok(LuaValue::Table(table))
        })
    }
}
