#![no_std]

macro_rules! userspace {
    ($name:ident, $feature:expr) => {
        #[cfg(feature = $feature)]
        mod $name;
        #[cfg(feature = $feature)]
        pub use $name::*;
    };
}

userspace!(k1, "k1");
userspace!(k2, "k2");
