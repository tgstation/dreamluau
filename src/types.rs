use constcat::concat_slices;
use meowtonin::ByondValue;

use crate::helpers::{GLOBALS, WORLD};

pub struct ValueTag;

#[allow(non_upper_case_globals)]
impl ValueTag {
    pub const Null: u8 = 0x00;
    pub const Turf: u8 = 0x01;
    pub const Obj: u8 = 0x02;
    pub const Mob: u8 = 0x03;
    pub const Area: u8 = 0x04;
    pub const Client: u8 = 0x05;
    pub const String: u8 = 0x06;

    pub const MobTypepath: u8 = 0x08;
    pub const ObjTypepath: u8 = 0x09;
    pub const TurfTypepath: u8 = 0x0A;
    pub const AreaTypepath: u8 = 0x0B;
    pub const Resource: u8 = 0x0C;
    pub const Image: u8 = 0x0D;
    pub const World: u8 = 0x0E;

    // Lists
    pub const List: u8 = 0x0F;
    pub const ArgList: u8 = 0x10;
    pub const MobContents: u8 = 0x17;
    pub const TurfContents: u8 = 0x18;
    pub const AreaContents: u8 = 0x19;
    pub const WorldContents: u8 = 0x1A;
    pub const ObjContents: u8 = 0x1C;
    pub const DatumTypepath: u8 = 0x20;
    pub const ProcRef: u8 = 0x26;
    pub const File: u8 = 0x27;
    pub const MobVars: u8 = 0x2C;
    pub const ObjVars: u8 = 0x2D;
    pub const TurfVars: u8 = 0x2E;
    pub const AreaVars: u8 = 0x2F;
    pub const ClientVars: u8 = 0x30;
    pub const Vars: u8 = 0x31;
    pub const MobOverlays: u8 = 0x32;
    pub const MobUnderlays: u8 = 0x33;
    pub const ObjOverlays: u8 = 0x34;
    pub const ObjUnderlays: u8 = 0x35;
    pub const TurfOverlays: u8 = 0x36;
    pub const TurfUnderlays: u8 = 0x37;
    pub const AreaOverlays: u8 = 0x38;
    pub const AreaUnderlays: u8 = 0x39;
    pub const ImageOverlays: u8 = 0x40;
    pub const ImageUnderlays: u8 = 0x41;
    pub const ImageVars: u8 = 0x42;
    pub const BinaryObject: u8 = 0x45;
    pub const TurfVisContents: u8 = 0x4B;
    pub const ObjVisContents: u8 = 0x4C;
    pub const MobVisContents: u8 = 0x4D;
    pub const TurfVisLocs: u8 = 0x4E;
    pub const ObjVisLocs: u8 = 0x4F;
    pub const MobVisLocs: u8 = 0x50;
    pub const WorldVars: u8 = 0x51;
    pub const GlobalVars: u8 = 0x52;
    pub const Filters: u8 = 0x53;
    pub const ImageVisContents: u8 = 0x54;

    pub const Datum: u8 = 0x21;
    pub const SaveFile: u8 = 0x23;

    pub const Number: u8 = 0x2A;
    pub const Appearance: u8 = 0x3A;
    pub const Pointer: u8 = 0x3C;
}

pub fn dm_type_name(t: u8) -> String {
    match t {
        ValueTag::Null => "null".to_string(),
        ValueTag::Turf => "turf".to_string(),
        ValueTag::Obj => "obj".to_string(),
        ValueTag::Mob => "mob".to_string(),
        ValueTag::Area => "area".to_string(),
        ValueTag::Client => "client".to_string(),
        ValueTag::String => "string".to_string(),
        ValueTag::MobTypepath => "mob typepath".to_string(),
        ValueTag::ObjTypepath => "obj typepath".to_string(),
        ValueTag::TurfTypepath => "turf typepath".to_string(),
        ValueTag::AreaTypepath => "area typepath".to_string(),
        ValueTag::Resource => "resource".to_string(),
        ValueTag::Image => "image".to_string(),
        ValueTag::World => "world".to_string(),
        ValueTag::List => "list".to_string(),
        ValueTag::ArgList => "arg list".to_string(),
        ValueTag::MobContents => "mob contents".to_string(),
        ValueTag::TurfContents => "turf contents".to_string(),
        ValueTag::AreaContents => "area contents".to_string(),
        ValueTag::WorldContents => "world contents".to_string(),
        ValueTag::ObjContents => "obj contents".to_string(),
        ValueTag::DatumTypepath => "datum typepath".to_string(),
        ValueTag::ProcRef => "proc reference".to_string(),
        ValueTag::File => "file".to_string(),
        ValueTag::MobVars => "mob vars".to_string(),
        ValueTag::ObjVars => "obj vars".to_string(),
        ValueTag::TurfVars => "turf vars".to_string(),
        ValueTag::AreaVars => "area vars".to_string(),
        ValueTag::ClientVars => "client vars".to_string(),
        ValueTag::Vars => "vars".to_string(),
        ValueTag::MobOverlays => "mob overlays".to_string(),
        ValueTag::MobUnderlays => "mob underlays".to_string(),
        ValueTag::ObjOverlays => "obj overlays".to_string(),
        ValueTag::ObjUnderlays => "obj underlays".to_string(),
        ValueTag::TurfOverlays => "turf overlays".to_string(),
        ValueTag::TurfUnderlays => "turf underlays".to_string(),
        ValueTag::AreaOverlays => "area overlays".to_string(),
        ValueTag::AreaUnderlays => "area underlays".to_string(),
        ValueTag::ImageOverlays => "image overlays".to_string(),
        ValueTag::ImageUnderlays => "image underlays".to_string(),
        ValueTag::ImageVars => "image vars".to_string(),
        ValueTag::BinaryObject => "binary object".to_string(),
        ValueTag::TurfVisContents => "turf vis_contents".to_string(),
        ValueTag::ObjVisContents => "obj vis_contents".to_string(),
        ValueTag::MobVisContents => "mob vis_contents".to_string(),
        ValueTag::TurfVisLocs => "turf vis_locs".to_string(),
        ValueTag::ObjVisLocs => "obj vis_locs".to_string(),
        ValueTag::MobVisLocs => "mob vis_locs".to_string(),
        ValueTag::WorldVars => "world vars".to_string(),
        ValueTag::GlobalVars => "global vars".to_string(),
        ValueTag::Filters => "filter(s)".to_string(),
        ValueTag::ImageVisContents => "image vis_contents".to_string(),
        ValueTag::Datum => "datum".to_string(),
        ValueTag::SaveFile => "savefile".to_string(),
        ValueTag::Number => "number".to_string(),
        ValueTag::Appearance => "appearance".to_string(),
        ValueTag::Pointer => "pointer".to_string(),
        other => format!("unknown ({})", other),
    }
}

pub fn type_name_for_obj(obj: &ByondValue) -> String {
    if *obj == GLOBALS {
        "global vars object".to_string()
    } else {
        dm_type_name(obj.get_type().0)
    }
}

/// Types that refer to a list of contents
pub const CONTENTS_TYPES: &[u8] = &[
    ValueTag::MobContents,
    ValueTag::TurfContents,
    ValueTag::AreaContents,
    ValueTag::WorldContents,
    ValueTag::ObjContents,
];

/// Types that refer to a list of vars
pub const VARS_TYPES: &[u8] = &[
    ValueTag::MobVars,
    ValueTag::ObjVars,
    ValueTag::TurfVars,
    ValueTag::AreaVars,
    ValueTag::ClientVars,
    ValueTag::Vars,
    ValueTag::ImageVars,
    ValueTag::WorldVars,
    ValueTag::GlobalVars,
];

/// Types that refer to a list of static appearances
pub const APPEARANCE_LIST_TYPES: &[u8] = &[
    ValueTag::MobOverlays,
    ValueTag::MobUnderlays,
    ValueTag::ObjOverlays,
    ValueTag::ObjUnderlays,
    ValueTag::TurfOverlays,
    ValueTag::TurfUnderlays,
    ValueTag::AreaOverlays,
    ValueTag::AreaUnderlays,
    ValueTag::ImageOverlays,
    ValueTag::ImageUnderlays,
];

/// Types that refer to a list of vis_contents
pub const VIS_CONTENTS_TYPES: &[u8] = &[
    ValueTag::TurfVisContents,
    ValueTag::ObjVisContents,
    ValueTag::MobVisContents,
    ValueTag::ImageVisContents,
];

/// Types that refer to a list of vis_locs
pub const VIS_LOCS_TYPES: &[u8] = &[
    ValueTag::TurfVisLocs,
    ValueTag::ObjVisLocs,
    ValueTag::MobVisLocs,
];

/// All types that are considered lists
pub const ALL_LIST_TYPES: &[u8] = concat_slices!([u8]:
    CONTENTS_TYPES,
    VARS_TYPES,
    APPEARANCE_LIST_TYPES,
    VIS_CONTENTS_TYPES,
    VIS_LOCS_TYPES,
    &[ValueTag::List, ValueTag::ArgList],
);

/// All types that are considered datums
pub const DATUM_TYPES: &[u8] = &[
    ValueTag::Turf,
    ValueTag::Obj,
    ValueTag::Mob,
    ValueTag::Area,
    ValueTag::Image,
    ValueTag::Datum,
];

/// Types that can be indexed by arbitrary strings
pub const STRING_INDEXABLE_TYPES: &[u8] = concat_slices!([u8]:
    VARS_TYPES,
    DATUM_TYPES,
    &[
        ValueTag::Client,
        ValueTag::List,
        ValueTag::ArgList,
        ValueTag::Appearance,
        ValueTag::World
    ],
);

/// Types that have procs that can be called (World is an exception because `global` has that tag, but cannot have procs defined)
pub const PROC_HAVING_TYPES: &[u8] = concat_slices!([u8]:
    ALL_LIST_TYPES,
    DATUM_TYPES,
    &[ValueTag::Client, ValueTag::List, ValueTag::ArgList],
);

/// Types that can have procs defined by the user (World is an exception because you can define procs for `world`, but not `global`, which both have that tag)
pub const PROC_DEFINABLE_TYPES: &[u8] = concat_slices!([u8]: DATUM_TYPES, &[ValueTag::Client]);

/// Types that can be indexed in some way
pub const INDEXABLE_TYPES: &[u8] =
    concat_slices!([u8]: PROC_HAVING_TYPES, &[ValueTag::Appearance, ValueTag::World]);

#[inline]
pub fn can_index_at_all(value: &ByondValue) -> bool {
    INDEXABLE_TYPES.contains(&value.get_type().0)
}

#[inline]
pub fn can_index_by_number(value: &ByondValue) -> bool {
    ALL_LIST_TYPES.contains(&value.get_type().0)
}

#[inline]
pub fn can_index_by_string(value: &ByondValue) -> bool {
    STRING_INDEXABLE_TYPES.contains(&value.get_type().0)
}

#[inline]
pub fn can_index_by_anything(value: &ByondValue) -> bool {
    value.get_type().0 == ValueTag::List
}

pub fn is_valid_var_index_for_value(var: &[u8], value: &ByondValue) -> bool {
    let value_type = value.get_type().0;
    STRING_INDEXABLE_TYPES.contains(&value_type)
        || *value == GLOBALS
        || std::str::from_utf8(var).is_ok_and(|var_str| {
            (value_type == ValueTag::SaveFile
                && matches!(
                    var_str,
                    "byond_build" | "byond_version" | "cd" | "dir" | "eof" | "name"
                ))
                || (*value == WORLD
                    && matches!(
                        var_str,
                        "address"
                            | "area"
                            | "byond_build"
                            | "byond_version"
                            | "cache_lifespan"
                            | "contents"
                            | "cpu"
                            | "executor"
                            | "fps"
                            | "game_state"
                            | "host"
                            | "hub"
                            | "hub_password"
                            | "icon_size"
                            | "internet_address"
                            | "log"
                            | "loop_checks"
                            | "map_format"
                            | "map_cpu"
                            | "maxx"
                            | "maxy"
                            | "maxz"
                            | "mob"
                            | "movement_mode"
                            | "name"
                            | "params"
                            | "port"
                            | "process"
                            | "realtime"
                            | "reachable"
                            | "sleep_offline"
                            | "status"
                            | "system_type"
                            | "tick_lag"
                            | "tick_usage"
                            | "time"
                            | "timeofday"
                            | "timezone"
                            | "turf"
                            | "url"
                            | "vars"
                            | "version"
                            | "view"
                            | "visibility"
                    ))
        })
}

pub fn is_valid_proc_index_for_value(proc: &str, value: &ByondValue) -> bool {
    let value_type = value.get_type().0;
    PROC_DEFINABLE_TYPES.contains(&value_type)
        || *value == WORLD
        || (ALL_LIST_TYPES.contains(&value_type)
            && matches!(
                proc,
                "Add"
                    | "Copy"
                    | "Cut"
                    | "Find"
                    | "Insert"
                    | "Join"
                    | "Remove"
                    | "RemoveAll"
                    | "Splice"
                    | "Swap"
            ))
}
