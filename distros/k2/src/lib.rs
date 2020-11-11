#![no_std]

use choochoos::sys;
use ts7200::bwprintln;

#[no_mangle]
pub extern "C" fn FirstUserTask() -> ! {
    bwprintln!("Hello from user space k2!");
    sys::r#yield();
    bwprintln!("Hello once again from user space k2!");
    sys::exit();
}
