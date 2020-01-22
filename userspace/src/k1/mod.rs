use ts7200::blocking_println;

use choochoos_sys::{create, my_parent_tid, my_tid, r#yield};

// Note to the TAs:
// We set up the stack such that the initial LR passed to a task points to the
// "Exit" syscall method. This enables omitting a trailing Exit() call, as it
// will be implicitly invoked once the task return.

#[no_mangle]
pub extern "C" fn OtherTask() {
    blocking_println!("MyTid={:?} MyParentTid={:?}", my_tid(), my_parent_tid());
    r#yield();
    blocking_println!("MyTid={:?} MyParentTid={:?}", my_tid(), my_parent_tid());
    // implicit exit() on return
}

#[no_mangle]
pub extern "C" fn FirstUserTask() {
    blocking_println!("Created: {:?}", create(3, OtherTask).unwrap());
    blocking_println!("Created: {:?}", create(3, OtherTask).unwrap());
    blocking_println!("Created: {:?}", create(5, OtherTask).unwrap());
    blocking_println!("Created: {:?}", create(5, OtherTask).unwrap());
    // implicit exit() on return
}
