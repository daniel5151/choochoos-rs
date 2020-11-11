# This hacky script enables running old C-based `choochoos` userspaces on the new choochoos-rs kernel.
#
# Don't use this!
# This is _not_ a robust script, and only works on my local dev machine!
#
# The biggest hack is definitely the `--allow-multiple-definition` flag, as Clang and GCC both provide copies of certain
# compiler intrinsics (e.g: memcpy, software floating point routines, etc...). While it seems to work okay, it could
# result in some "spooky" behavior, and shouldn't be considered the "right" way to run C code with the kernel.
mkdir -p ./bin/
cp ../choochoos/libuserspace.a ./bin/

RUSTFLAGS='-L /lib/arm-none-eabi/lib -L /lib/gcc/arm-none-eabi/9.2.1 -lstdc++ -lc -lgcc -Clink-arg=--allow-multiple-definition' \
make $@ \
    EXTRA_KERNEL_FEATURES=legacy-implicit-exit \
    DISTRO=extern_userspace \
    EXTERN_DISTRO=userspace \
