# `choochoo-rs`

_A safer CS 452 OS_

## Building

### Initial Setup

Building choochoo-rs requires using a nightly version of the Rust compiler.

At the moment, the linux.student.cs.uwaterloo.ca does not come pre-loaded with
a nightly Rust compiler, so an additional one-time setup script is required to
build choochoos on the student CS environment.

This script only needs to be run once. It will automatically install rustup (if
it is not detected to be installed already), and download the lastest nightly
toolchiain, along with some other misc. dependencies.

```bash
./initial_setup.sh
```

### Building

The recommended way to build choochoos is via the provided `Makefile`.

```bash
make k1 # or make DEBUG=1 k1 for a debug build
```

## Running

The result of running the Makefile is a loadable elf binary under the `bin`
directory. This can be copied into the tftp directory and loaded on to the
ts7200 as expected.

