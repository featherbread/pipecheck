`pipecheck` is a Rust `Write` wrapper that handles broken pipe errors by terminating the current process.
See [the crate documentation](https://docs.rs/pipecheck/latest/pipecheck/) for details.

## Maintenance and Future Work

`pipecheck` was first implemented as a private module in [xt](https://github.com/featherbread/xt)
in May 2023, then extracted into an independent crate in October 2025.
I expect it to require no further maintenance or changes for many years to come.

I would like to release a version 1.0.0 whenever I have the time and desire
to implement decent cross-platform automated tests for its behavior.
New major versions may then be released if Rust stabilizes new `Write` methods
that would benefit from an explicit wrapper implementation.
