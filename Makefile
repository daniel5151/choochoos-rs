PROJECT_NAME=choochoo-rs
TARGET=armv4-none-eabi
THUMB_TARGET=armv4-none-eabi.json

all: build

build:
	cargo xbuild --target $(THUMB_TARGET)

build-release:
	cargo xbuild --target $(THUMB_TARGET) --release

clean:
	cargo clean