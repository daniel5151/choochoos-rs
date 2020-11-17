# choochoos-abi

This crate is the "source of truth" for the choochoos kernel-userspace ABI, and is used on both sides of the kernel/userspace boundary to ensure that syscall numbers, signatures, errors, etc... are correctly matched.

The `choochoos` ABI is follows the `extern "C"` calling convention for passing arguments, and is 100% compatible with the [original C-based ABI](https://student.cs.uwaterloo.ca/~cs452/W20/assignments/kernel.html) used in CS 452 at the University of Waterloo, insofar as the C-ABI signatures and behaviors match (i.e: syscall numbers may vary between different student implementations).
