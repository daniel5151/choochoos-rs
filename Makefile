DISTRO := k1

TARGET=armv4-none-eabi.json

CARGO_FLAGS = --target $(TARGET) -Z unstable-options -Z build-std=core,alloc --out-dir=bin
ifndef DEBUG
	CARGO_FLAGS += --release
endif

# TODO: explore https://doc.rust-lang.org/rustc/linker-plugin-lto.html to shrink
# the resulting binary size (assuming that userspace is still being statically
# linked with choochoos-kernel)

all: kernel

.PHONY: kernel
kernel:
	cargo build --manifest-path userspace/Cargo.toml $(CARGO_FLAGS) --features "$(DISTRO)"
	# cheeky hack to work around the fact that Rust doesn't really support linking
	# multiple static libraries together.
	arm-none-eabi-objcopy bin/libuserspace.a --redefine-sym rust_begin_unwind=user_rust_begin_unwind

	cargo build --manifest-path choochoos-kernel/Cargo.toml $(CARGO_FLAGS)

clean:
	cargo clean
