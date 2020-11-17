# Userspace Distributions

These crates compile to standalone `lib<XYZ>.a` static libraries which export valid `extern "C" FirstUserTask()` and `extern "C" NameServerTask` symbols.

The static libraries can then be linked with `choochoos-kernel` to generate executable `.elf` binaries.

This is automatically handed by the Makefile.
