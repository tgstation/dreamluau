use meowtonin::{ByondResult, ByondValue, FromByond, ToByond};
pub use object::ByondObject;

mod conversion;
mod object;
pub use conversion::{
    convert_from_table, convert_to_table, safe_convert_from_table, ConversionVariant,
};

#[derive(Clone, Default, PartialEq, Eq, Hash)]
pub struct Value(pub ByondValue);

impl ToByond for Value {
    fn to_byond(&self) -> ByondResult<ByondValue> {
        Ok(self.0.clone())
    }
}

impl FromByond for Value {
    fn from_byond(value: &ByondValue) -> ByondResult<Self> {
        Ok(Self(value.clone()))
    }
}

impl<S: AsRef<str>> From<S> for Value {
    fn from(value: S) -> Self {
        Self(value.as_ref().to_byond().unwrap())
    }
}
