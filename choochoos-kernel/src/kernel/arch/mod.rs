// HACK: the `cfg(target_arch)` calls really aught to be here,
// but it breaks my editor's rust language support (it assumes target_arch =
// "x86"), so they're commented out for now.

// #[cfg(target_arch = "arm")]
mod arm;

// #[cfg(target_arch = "arm")]
pub use arm::*;
