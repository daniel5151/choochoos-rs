# `choochoo-rs`

_A safer CS 452 OS_

## Building

Building choochoo-rs requires using a nightly version of the Rust compiler and
the `arm-non-eabi` toolchain.

### Initial Setup

At the time of writing, the `linux.student.cs.uwaterloo.ca` environment does not
come pre-loaded with a nightly Rust compiler, so an additional one-time setup
script is required to build choochoos on the student CS environment.

This script only needs to be run once. It will automatically install rustup (if
it is not detected to be installed already), and download the lastest nightly
toolchiain, along with some other misc. dependencies.

```bash
./initial_setup.sh
```

### Building the Kernel

The recommended way to build choochoos is via the provided `Makefile`.

```bash
make DISTRO=k1 # add DEBUG=1 for a debug build
```

The resulting elf binary is output to `./bin/choochoos-kernel`.

## Running

The result of running the Makefile is a loadable elf binary under the `bin`
directory. This can be copied into the tftp directory and loaded on to the
ts7200 as expected.

If for whatever reason you're not one of the lucky folks with direct access to a
physical ts7200 machine, you can also emulate the kernel using
[`ts7200`](https://github.com/daniel5151/ts7200).
