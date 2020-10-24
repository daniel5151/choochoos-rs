use core::ptr;

use abi::{syscall::SyscallNo, Tid};

use crate::kernel::Kernel;
use crate::util::user_slice::{self, UserSlice, UserSliceMut};

use super::userstack::UserStack;

fn dispatch_yield(kernel: &mut Kernel, _stack: &mut UserStack) {
    kernel.syscall_yield()
}

fn dispatch_exit(kernel: &mut Kernel, _stack: &mut UserStack) {
    kernel.syscall_exit();
}

fn dispatch_my_tid(kernel: &mut Kernel, stack: &mut UserStack) {
    let ret = kernel.syscall_my_tid().raw();
    stack.inject_return_value(ret)
}

fn dispatch_my_parent_tid(kernel: &mut Kernel, stack: &mut UserStack) {
    let ret = match kernel.syscall_my_parent_tid() {
        Ok(tid) => tid.raw() as isize,
        Err(code) => code as isize,
    };

    stack.inject_return_value(ret)
}

fn dispatch_create(kernel: &mut Kernel, stack: &mut UserStack) {
    let mut args = stack.args();
    let priority = unsafe { args.extract::<isize>() };
    let function = unsafe { args.extract::<Option<unsafe extern "C" fn()>>() };

    let ret = match kernel.syscall_create(priority, function) {
        Ok(tid) => tid.raw() as isize,
        Err(code) => code as isize,
    };

    stack.inject_return_value(ret);
}

fn dispatch_reply(kernel: &mut Kernel, stack: &mut UserStack) {
    let mut args = stack.args();
    let tid = unsafe { args.extract::<Tid>() };
    let reply_ptr = unsafe { args.extract::<*mut u8>() };
    let reply_len = unsafe { args.extract::<usize>() };

    let reply = if reply_ptr.is_null() {
        UserSlice::empty()
    } else {
        unsafe { user_slice::from_raw_parts(ptr::NonNull::new_unchecked(reply_ptr), reply_len) }
    };

    let ret = match kernel.syscall_reply(tid, reply) {
        Ok(response_len) => response_len,
        Err(code) => code as usize,
    };

    stack.inject_return_value(ret)
}

fn dispatch_recieve(kernel: &mut Kernel, stack: &mut UserStack) {
    let mut args = stack.args();
    let sender_tid_dst = unsafe { args.extract::<*mut Tid>() };
    let msg_ptr = unsafe { args.extract::<*mut u8>() };
    let msg_len = unsafe { args.extract::<usize>() };

    let sender_tid_dst = if sender_tid_dst.is_null() {
        None
    } else {
        unsafe { Some(ptr::NonNull::new_unchecked(sender_tid_dst)) }
    };

    let msg = if msg_ptr.is_null() {
        UserSliceMut::empty()
    } else {
        unsafe { user_slice::from_raw_parts_mut(ptr::NonNull::new_unchecked(msg_ptr), msg_len) }
    };

    if let Some(response_len) = kernel.syscall_recieve(sender_tid_dst, msg) {
        stack.inject_return_value(response_len)
    };
}

fn dispatch_send(kernel: &mut Kernel, stack: &mut UserStack) {
    let mut args = stack.args();
    let receiver_tid = unsafe { args.extract::<Tid>() };
    let msg_ptr = unsafe { args.extract::<*mut u8>() };
    let msg_len = unsafe { args.extract::<usize>() };
    let reply_ptr = unsafe { args.extract::<*mut u8>() };
    let reply_len = unsafe { args.extract::<usize>() };

    let msg = if msg_ptr.is_null() {
        UserSlice::empty()
    } else {
        unsafe { user_slice::from_raw_parts(ptr::NonNull::new_unchecked(msg_ptr), msg_len) }
    };

    let reply = if reply_ptr.is_null() {
        UserSliceMut::empty()
    } else {
        unsafe { user_slice::from_raw_parts_mut(ptr::NonNull::new_unchecked(reply_ptr), reply_len) }
    };

    match kernel.syscall_send(receiver_tid, msg, reply) {
        Ok(()) => {} // return value injected as part of the `Reply` syscall
        Err(code) => stack.inject_return_value(code),
    };
}

/// Called by the _swi_handler assembly routine
#[no_mangle]
unsafe extern "C" fn handle_syscall(no: u8, sp: *mut UserStack) {
    let mut sp = ptr::NonNull::new(sp).expect("passed null sp to handle_syscall");
    let stack = sp.as_mut();

    let syscall_no = SyscallNo::from_u8(no).expect("invalid syscall");
    kdebug!("Called {:x?}", syscall_no);

    let kernel = match &mut crate::KERNEL {
        Some(kernel) => kernel,
        None => core::hint::unreachable_unchecked(),
    };

    match syscall_no {
        SyscallNo::Yield => dispatch_yield(kernel, stack),
        SyscallNo::Exit => dispatch_exit(kernel, stack),
        SyscallNo::MyParentTid => dispatch_my_parent_tid(kernel, stack),
        SyscallNo::MyTid => dispatch_my_tid(kernel, stack),
        SyscallNo::Create => dispatch_create(kernel, stack),
        SyscallNo::Send => dispatch_send(kernel, stack),
        SyscallNo::Receive => dispatch_recieve(kernel, stack),
        SyscallNo::Reply => dispatch_reply(kernel, stack),
        other => panic!("unimplemented syscall: {:?}", other),
    };
}

extern "C" {
    // implemented in asm.s
    pub fn _swi_handler();
}
