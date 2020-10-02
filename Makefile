TARGET=armv4-none-eabi
THUMB_TARGET=armv4-none-eabi.json

export PATH := /u/cs452/public/xdev/bin:$(PATH)

CARGO_FLAGS = --target $(THUMB_TARGET) -Z unstable-options -Z build-std=core,alloc --out-dir=bin
ifndef DEBUG
	CARGO_FLAGS += --release
endif

all: k1 # update for each assignment

.PHONY: k1
k1:
	cargo build --manifest-path choochoos-kernel/Cargo.toml $(CARGO_FLAGS) --features "k1"

.PHONY: k2
k2:
	cargo build --manifest-path choochoos-kernel/Cargo.toml $(CARGO_FLAGS) --features "k2"

clean:
	cargo clean
