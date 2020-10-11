RUSTFLAGS='-L /lib/arm-none-eabi/lib -lc' \
make \
    kernel \
    EXTRA_KERNEL_FEATURES=legacy-implicit-exit \
    CUSTOM_USERSPACE=../choochoos/libuserspace.a
