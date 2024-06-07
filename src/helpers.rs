use meowtonin::{
    sys::{ByondValueData, CByondValue},
    ByondValue,
};

use crate::types::ValueTag;

pub static WORLD: ByondValue = ByondValue(CByondValue {
    type_: ValueTag::World,
    junk1: 0,
    junk2: 0,
    junk3: 0,
    data: ByondValueData { ref_: 0 },
});

pub static GLOBALS: ByondValue = ByondValue(CByondValue {
    type_: ValueTag::World,
    junk1: 0,
    junk2: 0,
    junk3: 0,
    data: ByondValueData { ref_: 1 },
});
