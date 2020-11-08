DISTRO := k1
EXTRA_KERNEL_FEATURES :=
EXTRA_USER_FEATURES :=

CARGO_FLAGS = \
	--target armv4-none-eabi.json \
	-Z unstable-options \
	-Z build-std=core,alloc \
	--out-dir=bin
CARGO_KERNEL_FEATURES += $(EXTRA_KERNEL_FEATURES)
CARGO_USER_FEATURES += $(DISTRO) $(EXTRA_USER_FEATURES)

ifndef DEBUG
	CARGO_FLAGS += --release
endif

ifdef KDEBUG
	CARGO_KERNEL_FEATURES += kdebug
endif

all: userspace kernel # order matters here

# TODO: explore https://doc.rust-lang.org/rustc/linker-plugin-lto.html to shrink
# the resulting binary size (assuming that userspace is still being statically
# linked with choochoos-kernel)

.PHONY: userspace
userspace:
	cargo build \
		$(CARGO_FLAGS) \
		--manifest-path userspace/Cargo.toml \
		--features "$(CARGO_USER_FEATURES)"
	# cheeky hack to work around the fact that Rust doesn't really support
	# linking multiple static libraries together.
	arm-none-eabi-objcopy bin/libuserspace.a \
		--redefine-sym rust_begin_unwind=user_rust_begin_unwind

.PHONY: kernel
kernel:
ifdef CUSTOM_USERSPACE
	# HACK: this is a terrible kludgy solution, but I don't want to deal with build
	# system stuff right now.
	mkdir -p ./bin/
	cp $(CUSTOM_USERSPACE) ./bin/
endif

	cargo build \
		$(CARGO_FLAGS) \
		--manifest-path choochoos-kernel/Cargo.toml \
		--features "$(CARGO_KERNEL_FEATURES)"

.PHONY: doc
doc:
	cargo doc --no-deps --document-private-items --all-features

clean:
	rm -rf ./bin/
	cargo clean
