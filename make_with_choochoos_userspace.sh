RUSTFLAGS='-L /lib/arm-none-eabi/lib -L /lib/gcc/arm-none-eabi/9.2.1 -lstdc++ -lc -lgcc -Clink-arg=--allow-multiple-definition' \
make $@ \
    kernel \
    EXTRA_KERNEL_FEATURES=legacy-implicit-exit \
    CUSTOM_USERSPACE=../choochoos/libuserspace.a
