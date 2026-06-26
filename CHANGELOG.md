## v0.2.0 (Unreleased)

### Added

- **Automatic unblocking of SIGPIPE** by manipulating the current thread's
  signal mask in addition to the process-wide action. The thinking is that
  since pipecheck always terminates the process one way or another on broken
  pipe errors, it should strive to avoid the exit fallback in as many
  situations as it reasonably can.
- **Documentation regarding PID 1 on Linux**, for which pipecheck will fall
  back to an exit.

## v0.1.3 (2026-01-25)

### Changed

- **Created `src/pipecheck.rs`** with independent documentation and licensing
  info to facilitate easy vendoring into other projects.
- **Scoped `libc` dependency to `cfg(unix)`** to reflect how it's used.

## v0.1.2 (2026-01-20)

### Added

- **Documentation of important caveats** within the crate, including a possible
  (but sound) race condition and notes on signal masking.

### Changed

- **Using `sigaction` rather than `signal`** to reset the SIGPIPE handler,
  which is broadly considered more portable and better-defined for
  multi-threaded programs.

## v0.1.1 (2025-10-25)

### Added

- **The `pipecheck::wrap` constructor**, which I think reads more fluently than
  `pipecheck::Writer::new`.

## v0.1.0 (2025-10-25)

This is the initial release of `pipecheck`, a tiny piece of [xt][xt] that I
decided to extract into an independent crate. The implementation is exactly
what xt has used for years, plus one small pattern binding change for
compatibility all the way down to Rust 1.36.0.

[xt]: https://github.com/featherbread/xt
