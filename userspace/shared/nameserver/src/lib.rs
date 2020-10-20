#![no_std]

use owo_colors::OwoColorize;

use ts7200::bwprintln;

use choochoos_api as sys;

#[no_mangle]
pub extern "C" fn NameServerTask() {
    bwprintln!(
        "{}",
        "WARNING: Name Server is currently not implemented!".yellow()
    );
    sys::exit();
}
