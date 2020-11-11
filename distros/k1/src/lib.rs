#![feature(external_doc)]
#![no_std]

#[doc(include = "writeup.md")]
#[allow(unused_imports)]
pub mod writeup {
    use super::*;
}

use choochoos::sys;
use ts7200::bwprintln;

extern "C" fn other_task() -> ! {
    bwprintln!(
        COM2,
        "MyTid={:?} MyParentTid={:?}",
        sys::my_tid(),
        sys::my_parent_tid()
    );
    sys::r#yield();
    bwprintln!(
        COM2,
        "MyTid={:?} MyParentTid={:?}",
        sys::my_tid(),
        sys::my_parent_tid()
    );
    sys::exit();
}

// marked pub so that it can be referenced from the writeup docs.
pub extern "C" fn first_user_task() -> ! {
    bwprintln!(COM2, "Created: {:?}", sys::create(3, other_task).unwrap());
    bwprintln!(COM2, "Created: {:?}", sys::create(3, other_task).unwrap());
    bwprintln!(COM2, "Created: {:?}", sys::create(5, other_task).unwrap());
    bwprintln!(COM2, "Created: {:?}", sys::create(5, other_task).unwrap());
    bwprintln!(COM2, "FirstUserTask: exiting");
    sys::exit();
}

// FirstUserTask has a priority of 0, and our kernel doesn't support negative
// priorities. Thus, we need to have the FirstUserTask spawn a new "true"
// FirstUserTask with a higher priority.
#[no_mangle]
pub extern "C" fn FirstUserTask() -> ! {
    sys::create(4, first_user_task).unwrap();
    sys::exit();
}
