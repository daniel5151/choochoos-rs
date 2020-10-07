# Shared Userspace Tasks

e.g: UART server, Clock server, etc...

These crates do not export a `extern "C" fn FirstUserTask()`, and must be used as part of a userspace distro.
