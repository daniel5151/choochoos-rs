# `choochoo-rs`

_A safer CS 452 OS_

## Building

TODO: provide a build script to make things Just Work on the student CS environment.

### Prerequisites:

- [`rustup`](https://rustup.rs/) is set up.
- The `gcc-arm-none-eabi` toolchain is in your path

### Initial Setup

These commands only have to be run once.

```bash
rustup toolchain add nightly
rustup component add rust-src
cargo install cargo-xbuild
```

### Building

At the moment, the recomended way to build is via the provided `Makefile`

```bash
make k1 # or make DEBUG=1 k1 for a debug build
```
