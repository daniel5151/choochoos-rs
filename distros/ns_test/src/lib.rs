#![no_std]

use choochoos::{ns, sys};
use ts7200::bwprintln;

extern "C" fn task_1() -> ! {
    ns::register_as("Task1").unwrap();
    sys::exit();
}

extern "C" fn task_2() -> ! {
    ns::register_as("TASK_2").unwrap();
    sys::exit();
}

extern "C" fn task_3() -> ! {
    ns::register_as("task 3!!!").unwrap();
    sys::exit();
}

#[no_mangle]
pub extern "C" fn FirstUserTask() -> ! {
    let t1 = sys::create(1, task_1).unwrap();

    assert_eq!(ns::who_is("Task1").unwrap(), Some(t1));
    assert_eq!(ns::who_is("TASK_2").unwrap(), None);
    assert_eq!(ns::who_is("task 3!!!").unwrap(), None);

    // task 1 should have already died, which means it's TID will be recycled, and
    // it's registration overwritten.

    let t2 = sys::create(1, task_2).unwrap();
    let t3 = sys::create(1, task_3).unwrap();

    assert_eq!(ns::who_is("Task1").unwrap(), None);
    assert_eq!(ns::who_is("TASK_2").unwrap(), Some(t2));
    assert_eq!(ns::who_is("task 3!!!").unwrap(), Some(t3));
    assert_eq!(ns::who_is("???").unwrap(), None);

    bwprintln!("OK");

    sys::exit();
}
