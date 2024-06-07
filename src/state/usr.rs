use std::cell::RefCell;

use dreamluau_proc_macro::map_statics;
use meowtonin::{byond_fn, ByondValue};

thread_local! {
    static USR: RefCell<Option<ByondValue>> = RefCell::new(None);
    static USR_STACK: RefCell<Vec<Option<ByondValue>>> = RefCell::new(vec![]);
}

/// Byondapi does not currently support natively getting `usr` from DM.
/// As such, `usr` should be passed into this function from DM to ensure it can be read from lua.
#[map_statics(mut USR)]
#[byond_fn]
pub fn set_usr(new_usr: ByondValue) {
    usr.replace(new_usr);
}

#[map_statics(mut USR, mut USR_STACK)]
pub fn push_usr() {
    usr_stack.push(usr.take())
}

#[map_statics(mut USR_STACK)]
pub fn pop_usr() {
    usr_stack.pop();
}

#[map_statics(USR_STACK)]
pub fn peek_usr() -> Option<ByondValue> {
    usr_stack.iter().filter_map(|opt| opt.clone()).last()
}
