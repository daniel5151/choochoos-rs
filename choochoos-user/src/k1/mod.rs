use ts7200::blocking_println;

use choochoos_api::{create, exit, my_parent_tid, my_tid, r#yield};

// Note to the TAs:
// We set up the stack such that the initial LR passed to a task points to the
// "Exit" syscall method. This enables omitting a trailing Exit() call, as it
// will be implicitly invoked once the task return.

#[no_mangle]
pub extern "C" fn OtherTask() {
    blocking_println!("MyTid={:?} MyParentTid={:?}", my_tid(), my_parent_tid());
    r#yield();
    blocking_println!("MyTid={:?} MyParentTid={:?}", my_tid(), my_parent_tid());
    exit();
}

#[no_mangle]
pub extern "C" fn TrueFirstUserTask() {
    blocking_println!("Created: {:?}", create(3, OtherTask).unwrap());
    blocking_println!("Created: {:?}", create(3, OtherTask).unwrap());
    blocking_println!("Created: {:?}", create(5, OtherTask).unwrap());
    blocking_println!("Created: {:?}", create(5, OtherTask).unwrap());
    blocking_println!("FirstUserTask: exiting");
    exit();
}

// FirstUserTask has a priority of 0, and our kernel doesn't support negative
// priorities. Thus, we need to have the FirstUserTask spawn a new "true"
// FirstUserTask with a higher priority.
#[no_mangle]
pub extern "C" fn FirstUserTask() {
    create(4, TrueFirstUserTask).unwrap();
    exit();
}
