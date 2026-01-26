//! Cross-platform Unix-style handling of broken pipe errors.
//!
//! When any call to its underlying writer returns a [`BrokenPipe`](std::io::ErrorKind::BrokenPipe)
//! error, a [`Writer`] terminates the current process with a SIGPIPE signal, or falls back to a
//! plain exit with code 1.
//!
//! # Caveats
//!
//! On Unix, [`Writer`] works by resetting the process-wide SIGPIPE handler to its default behavior
//! immediately before sending SIGPIPE to the current thread. It may fall back to a plain exit:
//!
//!   * If it fails to reset the default SIGPIPE handler, which should never happen on a
//!     well-behaved system.
//!   * If the current thread's signal mask blocks delivery of SIGPIPE, which may happen if the
//!     current thread or its creator previously manipulated the signal mask. This version of
//!     `Writer` never unblocks SIGPIPE on its own (though a future version might).
//!   * If a racing thread installs a non-default SIGPIPE handler, in which case `Writer` may
//!     invoke that handler before exiting. This is _not_ considered unsound in terms of Rust's
//!     safety guarantees, as POSIX.1 prohibits the involved C library functions from being prone
//!     to data races or similar undefined behavior.
//!
//! Non-Unix platforms always fall back to a plain exit.
//!
//! # Why is this useful?
//!
//! Within a shell pipeline, it's good form for a process to exit quickly and silently as soon as
//! its downstream stops accepting input. Unix simplifies this with the SIGPIPE signal: when a
//! process writes to a pipe where all file descriptors referring to the read end have been closed,
//! the system delivers it a SIGPIPE, which by default terminates it.
//!
//! The existence of SIGPIPE introduces two challenges:
//!
//!   1. It's Unix-specific, so portable CLIs can't rely on it completely.
//!   2. A networked server can generate SIGPIPE on writes to a socket whose client has closed its
//!      read end, and terminating the server would break other clients' connections.
//!
//! Given these challenges, the Rust developers chose to override Unix's default behavior by
//! ignoring SIGPIPE before calling `main`, so that writes to broken pipes return a plain
//! [`BrokenPipe`](std::io::ErrorKind::BrokenPipe) error on all platforms.
//!
//! However, real-world Rust libraries don't always expose enough detail to easily distinguish
//! broken pipes from other errors. For example, the [`source`](std::error::Error::source)
//! implementation for a custom error might not expose an underlying [`io::Error`](std::io::Error)
//! even when traversing the entire chain of sources, which is problematic when error values are
//! coalesced into a `Box<dyn Error>` (or similar) and passed up the call stack.
//!
//! [`Writer`] instead plumbs its logic into every write, catching broken pipe errors and
//! terminating the process before they can be lost or obscured. Compared to modifying the
//! process-wide SIGPIPE behavior at the start of a Rust program, this approach is more
//! cross-platform and better scoped to the specific writes where termination is desired
//! (generally standard output and error streams).
//!
//! Note that termination on Unix attempts to use the real default behavior of SIGPIPE; `Writer`
//! does not employ incorrect hacks like exiting with code 141 (mimicking the shell return code of
//! a process terminated by SIGPIPE).
//!
//! # Can I avoid adding such a small crate to my supply chain?
//!
//! [`src/pipecheck.rs`](../src/pipecheck/pipecheck.rs.html) contains the entire implementation of
//! the crate with independent documentation and licensing information, with the explicit goal of
//! easy copy-paste vendoring into your own codebase.
//!
//! You will need to depend on the `libc` crate (for at least `cfg(unix)`), add `mod pipecheck;`
//! to your crate root or an appropriate parent module, and ensure that your lint settings allow
//! the module's unsafe code.
//!
//! # Further Reading
//!
//! For further background on SIGPIPE, Rust's handling of it, and cross-platform portability
//! concerns surrounding broken pipes, see:
//!
//! - <https://github.com/rust-lang/rust/issues/62569>
//! - <https://stackoverflow.com/a/65760807>
//! - <https://github.com/BurntSushi/ripgrep/issues/200#issuecomment-616884727>
//!
//! The concept of `pipecheck` was inspired by Go's default behavior for broken pipes: terminating
//! the program if the write was to a standard output or error stream, and otherwise returning a
//! plain error. For background on Go's behavior and runtime implementation, see:
//!
//! - <https://pkg.go.dev/os/signal#hdr-SIGPIPE>
//! - <https://cs.opensource.google/go/go/+/refs/tags/go1.23.6:src/os/file_unix.go;l=252>
//! - <https://cs.opensource.google/go/go/+/refs/tags/go1.23.6:src/runtime/signal_unix.go;l=333>

mod pipecheck;

pub use pipecheck::{wrap, Writer};
