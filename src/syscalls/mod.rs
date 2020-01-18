mod create;
mod exit;
mod r#yield;

pub use create::create;
pub use exit::exit;
pub use r#yield::r#yield;

pub enum Syscall {
    Yield,
    Exit,
    MyTid,
    MyParentTid,
    Create {
        priority: isize,
        function: Option<extern "C" fn()>,
    },
}
