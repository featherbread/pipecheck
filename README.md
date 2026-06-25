`pipecheck` is a Rust `Write` wrapper that handles broken pipe errors by terminating the current process.
See [the crate documentation](https://docs.rs/pipecheck/latest/pipecheck/) for details.

## History and Future Work

`pipecheck` was first implemented as a private module in [xt](https://github.com/featherbread/xt)
in May 2023, then extracted into an independent crate in October 2025.

As of June 2026, I'm considering customization of the fallback exit code as a
potential future improvement.

With that, and with improved cross-platform testing in place, it would be nice
to ship a v1.0.0 and only release new versions if Rust stabilizes new `Write`
methods or gaps in platform support are found (the crate is so simple there's
little room for feature work). However, this is a very low-priority side project.
