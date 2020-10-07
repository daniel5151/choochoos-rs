#![no_std]

use ts7200::blocking_println;

use choochoos_api as sys;

#[no_mangle]
pub extern "C" fn FirstUserTask() {
    blocking_println!("Hello from user space k2!");
    sys::r#yield();
    blocking_println!("Hello once again from user space k2!");
    sys::exit();
}
