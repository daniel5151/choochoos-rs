/// Mirrors the core::ffi::c_void type, but adding a Copy derive
#[repr(u8)]
#[derive(Clone, Copy)]
#[allow(dead_code)]
pub enum Void {
    #[doc(hidden)]
    Variant1,
    #[doc(hidden)]
    Variant2,
}
