# choochoos

The choochoos userspace library.

Provides common choochoos functionality shared across all userspace distros. e.g: syscall implementations, panic handler, nameserver implementation, etc...

In addition to providing an idiomatic Rust interface to `choochoos`, this library also exports a C API modeled after the [CS 452 Kernel Specification](https://student.cs.uwaterloo.ca/~cs452/W20/assignments/kernel.html).
