use ts7200::blocking_println;

use choochoos_sys::*;

fn str_from_null_terminated_bytes(buf: &[u8]) -> Result<&str, core::str::Utf8Error> {
    let n = buf.iter().position(|&x| x == b'\0').unwrap_or(buf.len());
    core::str::from_utf8(&buf[..n])
}

pub extern "C" fn PongTask() {
    let mut buf = [0u8; 32];
    let tid = receive(&mut buf[..]).unwrap();
    let msg = str_from_null_terminated_bytes(&buf).unwrap();

    blocking_println!("pong received tid={:?} msg={:?}", tid, msg);
}

#[no_mangle]
pub extern "C" fn FirstUserTask() {
    let tid1 = create(3, PongTask).unwrap();
    let tid2 = create(5, PongTask).unwrap();

    send(tid1, b"ping1").unwrap();
    send(tid2, b"ping2").unwrap();

    // implicit Exit() on return
}
