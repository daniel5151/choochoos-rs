use core::ffi::c_void;

// provided by the linker
extern "C" {
    static __USER_STACKS_START__: c_void;
}

const USER_STACK_SIZE: usize = 0x40000;

/// Helper POD struct to init new user task stacks
#[repr(C)]
#[derive(Debug)]
struct FreshStack {
    dummy_syscall_response: usize,
    start_addr: extern "C" fn(),
    spsr: usize,
    regs: [usize; 13],
    lr: fn(),
}

pub fn create(
    scheduler: &mut crate::Scheduler,
    priority: isize,
    function: Option<extern "C" fn()>,
) -> isize {
    let function = match function {
        Some(f) => f,
        // TODO? make this an error code?
        None => panic!("Cannot create task with null pointer"),
    };

    // get a fresh td (and corresponding tid) from the scheduler
    let (tid, sp) = scheduler.new_task(priority);

    // set the td's sp to point the new stack
    let start_of_stack = unsafe {
        (&__USER_STACKS_START__ as *const _ as usize) + (USER_STACK_SIZE * (tid.raw() + 1))
    };
    *sp = (start_of_stack - core::mem::size_of::<FreshStack>()) as *mut usize;

    // set up memory for the initial user stack
    let stackview = unsafe { &mut *(*sp as *mut FreshStack) };
    stackview.dummy_syscall_response = 0xdead_beef;
    stackview.spsr = 0xc0;
    stackview.start_addr = function;
    for r in &mut stackview.regs {
        *r = 0;
    }
    stackview.lr = choochoos_sys::exit;

    tid.raw() as isize
}
