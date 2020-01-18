PROJECT_NAME=choochoo-rs
TARGET=armv4-none-eabi
THUMB_TARGET=armv4-none-eabi.json

CARGO_FLAGS = --target $(THUMB_TARGET)
ifndef DEBUG
	CARGO_FLAGS += --release
endif

all: build

build:
	cargo xbuild $(CARGO_FLAGS)

.PHONY: k1
k1:
	cargo xbuild $(CARGO_FLAGS) --features "k1"

clean:
	cargo clean
