PROJECT_NAME=choochoo-rs
TARGET=armv4-none-eabi
THUMB_TARGET=armv4-none-eabi.json

CARGO_FLAGS = --target $(THUMB_TARGET)
ifndef DEBUG
	CARGO_FLAGS += --release
endif

all: k1

.PHONY: k1
k1:
	cargo xbuild $(CARGO_FLAGS) --features "k1"

.PHONY: k2
k2:
	cargo xbuild $(CARGO_FLAGS) --features "k2"

clean:
	cargo clean
