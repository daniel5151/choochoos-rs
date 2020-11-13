# `choochoo-rs`

_A Rusty CS 452 OS_

## Building

Building choochoo-rs requires using a nightly version of the Rust compiler.

Notably, the `arm-none-eabi` toolchain does _not_ need to be installed, as all
low-level assembly routines are written using Rust's `#[naked]` functions and
inline `asm!` macro.

### Initial Setup

At the time of writing, the `linux.student.cs.uwaterloo.ca` environment does not
come pre-loaded with an up-to-date nightly Rust compiler, so an additional
one-time setup script is required to build choochoos-rs on the student CS
environment.

This script only needs to be run once. It will automatically install rustup (the
official Rust installer and toolchain manager), and download the latest nightly
toolchain.

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
TS-7200 as expected.

You can also run the OS on [`ts7200`](https://github.com/daniel5151/ts7200), my
TS-7200 emulator.

## Documentation

The kernel is documented using Rust's incredibly powerful built-in inline
documentation capabilities. Running `make doc` will generate rich HTML
documentation under the `/target/doc`.

A good starting point would be to open `/target/doc/k1/writeup/index.html`,
which includes a brief overview of how rustdoc works, and why it's awesome for
documenting a Kernel.

## Use of Nightly Features

The core OS implementation only requires two nightly features:

-   [asm](https://doc.rust-lang.org/unstable-book/library-features/asm.html)
-   [naked_functions](https://doc.rust-lang.org/unstable-book/language-features/naked-functions.html)

These could be removed by having separate, standalone `.asm` files in the repo,
though that would require installing a separate `arm-none-eabi` toolchain, which
seems like a lot of hassle.

Aside from those two key features, there are several non-critical docs related
nightly features are used as well:

-   [external_doc](https://doc.rust-lang.org/unstable-book/language-features/external-doc.html):
    -   Allows assignment writeups to live in a separate `.md` file, instead of
        having to be written as an inline top-level doc-comment.
-   [doc_cfg](https://doc.rust-lang.org/unstable-book/language-features/doc-cfg.html):
    -   Quality of Life feature that highlights when certain items require a
        cargo feature to be enabled (e.g: kernel heap support).

## Cool Rust Features

This is a (very) non-exhaustive list of cool features that Rust brings to the
table when it comes to kernel development.

#### Type-level enforcement that Tasks call `Exit()`.

While most C compilers implement some sort of `__attribute__((noreturn))`
attribute, and the C++11 standard specifies a `[[noreturn]]` attribute, these
attributes aren't type-level constructs, and can't be used to enforce
non-returning computation through
[function pointers](https://stackoverflow.com/questions/28739082/how-to-use-noreturn-with-function-pointer).

As such, users must be incredibly careful to make sure that any task they write
in C/C++ call `Exit()` when they terminate, as forgetting to do so will result
in undefined behavior (typically causing execution to jump to some random point
in memory, and often resulting in an access violation).

Rust's counterpart to the C/C++ notion of `noreturn` is the
[`never`](https://doc.rust-lang.org/std/primitive.never.html) type (written as
`!`), which represents a computation that never resolves to a value. Since it's
part of the type system, `!` doesn't suffer the same limitations as the
`noreturn` attribute in C/C++, and can be used to enforce non-returning
computation through function pointers, and catch errors at compile time!

For example: the type of `choochoos::abi::syscall::signature::Create` is defined
to only accept function pointers with the type `Option<extern "C" fn() -> !>`
(i.e: a nullable function pointer which takes no parameters, and never returns).
This type is distinct and incompatible with `Option<extern "C" fn()>`, which
would be a function that returns `()` (the equivalent of `void` in C/C++).

For example: If a user forgets to include a call to `choochoos::sys::exit()` at
the end of a task, or happens to omit the `!` return type from the task's
function definition, their code won't even compile!
