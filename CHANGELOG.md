## Unreleased

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
