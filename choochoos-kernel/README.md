# choochoos-kernel

The core `choochoos` kernel implementation.

Implements the [CS 452 Kernel Specification](https://student.cs.uwaterloo.ca/~cs452/W20/assignments/kernel.html).

The crate is structured to support multiple architectures, with all architecture and platform specific code isolated from the generic platform-agnostic kernel implementation.

At the moment, only `arch:arm` (32 bit ARM) on `platform:ts7200` is implemented.
