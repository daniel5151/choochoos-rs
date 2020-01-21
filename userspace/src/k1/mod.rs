use ts7200::blocking_println;

#[no_mangle]
pub extern "C" fn init_task() {
    blocking_println!("Hello from user space!");
    choochoos_sys::r#yield();
    blocking_println!("Hello once again from user space!");
    // implicit Exit() on return
}
