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

### Building

The recommended way to build choochoos is via the provided `Makefile`.

The `DISTRO` makevar specifies which userspace "distribution" should be built.
For example, passing `DISTRO=k1` will build `./userspace/k1`.

```bash
make DISTRO=k1 # add DEBUG=1 for a debug build
```

The resulting elf binary is output to `./bin/choochoos-kernel`.

#### (optional) Using an External Userspace (e.g: written in C/C++)

The `choochoos` kernel can link with arbitrary static libraries located in the
`./bin` directory, so long as they export a valid `extern "C" FirstUserTask`
function.

There are two different ways of linking with the `choochoos` kernel:

1. Using the built-in `NameServerTask` and C API
    - The `choochoos` userspace library exports a C API matching the
      [CS 452 Kernel Specification](https://student.cs.uwaterloo.ca/~cs452/W20/assignments/kernel.html),
      which can be called with the appropriate header file. It also includes a
      pre-defined `NameServerTask`.
2. Directly linking with the `chooochoos` kernel.
    - This bypasses the built-in `NameServerTask` and C API, and requires that
      the static library provide the appropriate methods.
    - _WARNING:_ The static library must use the same syscall numbers and
      calling convention as expected by the `choochoos` kernel, or else things
      will not work!

e.g: to link with `./bin/libfoo.a`:

-   Option 1: `make DISTRO=external_userspace EXTERN_DISTRO=foo`
-   Option 2: `make DISTRO=foo`

It is highly recommended to link with the `choochoos` userspace library, as this
minimizes the chance of mismatched syscall issues.

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

## External Dependencies

One of Rust's greatest strengths is its package management system (`cargo`),
which makes it easy to download and use external dependencies.

The following dependencies aren't strictly "required", and could certainly be
re-implemented "in-tree" if necessary.

-   [`bit_field`](https://crates.io/crates/bit_field) - Basic utilities for
    working with bitfields in Rust.
-   [`bstr`](https://crates.io/crates/bstr) - A string type that is not required
    to be valid UTF-8.
    -   Primarily useful when debugging `[u8]` buffers
-   [`heapless`](https://crates.io/crates/heapless) - Data structures that don't
    require dynamic memory allocation.
    -   The syntax is a bit weird, as `heapless` doesn't use Rust's (still in
        development)
        [const generics](https://rust-lang.github.io/rfcs/2000-const-generics.html)
        feature, and instead [ab]uses the type system to emulate const-generics.
    -   Rust's built-in [`alloc`](https://doc.rust-lang.org/alloc/) crate
        includes all the generic data structures one would expect, like Vectors,
        Heaps, Maps, Sets, etc... Unfortunately, these data structures require a
        global allocator to be present.
    -   Alternatively, enabling the `heap` feature will use the
        [`linked_list_allocator`](https://crates.io/crates/linked_list_allocator)
        crate to set up a global allocator, thereby allowing standard Rust
        `alloc` types to be used instead.
        -   Note that unlike C++'s arcane custom allocator interface, Rust's
            [`GlobalAlloc`](https://doc.rust-lang.org/alloc/alloc/trait.GlobalAlloc.html)
            trait is very easy to implement, so if you don't care about
            `free`ing used memory and don't want to pull in any external
            dependencies, a trivial bump allocator can be implemented in less
            than a dozen lines of code.
-   [`owo-colors`](https://crates.io/crates/owo-colors) - Ergonomic abstraction
    over terminal color escape sequences.

Aside from these relatively lightweight dependencies, there are some heavier
(userspace-only) dependencies that provide some additional quality-of-life
features when writing `choochoos` programs. As always, these are not strictly
necessary, but are nonetheless very useful to have.

-   [`serde`](https://crates.io/crates/serde) - A framework for serializing and
    deserializing Rust data structures efficiently and generically.
    -   Serde is a staple of the Rust ecosystem, and is used by nearly every
        Rust project for de/serializing structured data.
    -   The typical C approach of casting a raw `struct` to a `char*` array is
        fraught with numerous pitfalls, the worst of which being the classic
        blunder of trying to send a `const char*` to another thread by reference
        instead of by value (i.e: by using a `char buf [MAX_SIZE]` instead).
    -   In `choochoos`, `serde` is used by the `serde-srr` abstraction (in the
        `choochoos` userspace library) to support safely and efficiently sending
        Rust types between Tasks. By leveraging Rust's powerful macro system,
        it's possible to automatically [derive](https://serde.rs/derive.html)
        the necessary de/serialization logic required to send/receive arbitrary
        types between tasks, saving oneself the hassle of manually coming up
        with ad-hoc de/serialization protocols when sending data between tasks.
-   [`postcard`](https://github.com/jamesmunns/postcard) - A no_std + serde
    compatible message library for Rust.
    -   While `serde` provides the _framework_ for de/serializing data, it's up
        to the individual implementation to specify the _protocol_ that data
        should be de/serialized to. For web-based applications, this might be
        `JSON`, configuration data might use `YAML` or `TOML`.
    -   `postcard` is a _binary_ format, and tries to use as compact of a data
        representation as possible. This makes it perfect for sending data
        between `choochoos` tasks, as it would be wasteful to send something
        like `JSON` between tasks directly.

## Cool Rust Features

This is a (very) non-exhaustive list of cool features that Rust brings to the
table when it comes to kernel development.

### Rust's "Split" Standard Library

I've actually written an entire blog post about this, partly inspired by my
experiences in CS 452:
[The C Standard Library is Not Dependency Free](https://prilik.com/blog/post/c-is-not-dependency-free/).

The gist of it is that Rust's Standard Library is very clearly divided into an
"embedded-safe" portion (called `core`), and a "required an OS" portion (called
`std`), which makes it very difficult (if not outright impossible) to
"accidentally" use a standard library routine that relies on some sort of
syscall (e.g: calling `malloc` requires `sbrk` to be defined).

### Type-level enforcement that Tasks call `Exit()`.

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
