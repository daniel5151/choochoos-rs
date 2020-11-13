//! Syscall handler implementations. See the [`Kernel`](crate::kernel::Kernel)
//! docs.

mod await_event;
mod create;
mod exit;
mod my_parent_tid;
mod my_tid;
mod perf;
mod receive;
mod reply;
mod send;
mod shutdown;
mod r#yield;
