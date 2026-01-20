//! Cross-platform Unix-style handling of broken pipe errors.
//!
//! When any call to its underlying writer returns a [`BrokenPipe`](io::ErrorKind::BrokenPipe)
//! error, a [`Writer`] terminates the current process with a SIGPIPE signal, or exits with code 1
//! on non-Unix systems.
//!
//! # Why is this useful?
//!
//! When a process runs in a Unix shell pipeline, it's good form for the process to exit quickly
//! and silently as soon as its downstream stops accepting input. Unix simplifies this with the
//! SIGPIPE signal: when a process writes to a pipe where all file descriptors referring to the
//! read end have been closed, the system sends it this signal, which by default terminates it.
//!
//! The existence of SIGPIPE introduces two challenges. First, it's Unix-specific, so portable CLIs
//! might not be able to rely on it. Second, a networked server can generate SIGPIPE when writing
//! to a socket whose client has closed its read end, and terminating the server would break other
//! clients' connections. Given these challenges, the Rust developers chose to override Unix's
//! default behavior by globally ignoring SIGPIPE prior to calling `main`, causing all writes to
//! broken pipes to return a plain [`BrokenPipe`](io::ErrorKind::BrokenPipe) error.
//!
//! Unfortunately, a well-meaning CLI that wants to handle broken pipes with a silent exit might
//! find it difficult using error values alone. Experience shows that real-world Rust libraries
//! don't always expose enough detail to easily distinguish this from other errors. For example,
//! the [`source`](std::error::Error::source) implementation in a library's custom error type might
//! not expose an underlying [`io::Error`] even when traversing the entire chain of sources, which
//! is especially problematic when the error type is coalesced into a `Box<dyn Error>` or similar.
//!
//! [`Writer`] instead plumbs this handling directly into every write operation, catching broken
//! pipe errors and terminating the process before anything else in the call stack has a chance to
//! obscure them. Unlike an up-front modification of the process-wide SIGPIPE behavior, this
//! approach is more cross-platform and better scoped to the specific writes where termination is
//! desired (generally on standard output and error streams).
//!
//! Note that termination on Unix invokes the real default behavior of SIGPIPE; `Writer` does not
//! employ incorrect hacks like exiting with code 141 (mimicking the shell return code of a process
//! terminated by SIGPIPE).
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
//! The concept of `pipecheck` was directly inspired by Go's default behavior for broken pipes:
//! terminating the program if the write was to a standard output or error stream, and otherwise
//! returning a plain error. For background on Go's behavior and runtime implementation, see:
//!
//! - <https://pkg.go.dev/os/signal#hdr-SIGPIPE>
//! - <https://cs.opensource.google/go/go/+/refs/tags/go1.23.6:src/os/file_unix.go;l=252>
//! - <https://cs.opensource.google/go/go/+/refs/tags/go1.23.6:src/runtime/signal_unix.go;l=333>

use std::io::{self, Write};

/// A convenient alias for [`Writer::new`].
pub fn wrap<W: Write>(w: W) -> Writer<W> {
    Writer::new(w)
}

/// A writer that silently terminates the program on broken pipe errors.
///
/// When any call to its underlying writer returns a [`BrokenPipe`](io::ErrorKind::BrokenPipe)
/// error, a [`Writer`] terminates the current process with a SIGPIPE signal, or exits with code 1
/// on non-Unix systems.
///
/// See [the crate documentation](crate) for more details.
pub struct Writer<W>(W)
where
    W: Write;

impl<W> Writer<W>
where
    W: Write,
{
    pub fn new(w: W) -> Writer<W> {
        Writer(w)
    }
}

impl<W> Write for Writer<W>
where
    W: Write,
{
    // Rust 1.0.0 includes the following methods.

    fn write(&mut self, buf: &[u8]) -> io::Result<usize> {
        check_for_broken_pipe(self.0.write(buf))
    }

    fn flush(&mut self) -> io::Result<()> {
        check_for_broken_pipe(self.0.flush())
    }

    fn write_all(&mut self, buf: &[u8]) -> io::Result<()> {
        check_for_broken_pipe(self.0.write_all(buf))
    }

    fn write_fmt(&mut self, fmt: std::fmt::Arguments<'_>) -> io::Result<()> {
        check_for_broken_pipe(self.0.write_fmt(fmt))
    }

    // Rust 1.36.0 stabilizes write_vectored.

    fn write_vectored(&mut self, bufs: &[io::IoSlice<'_>]) -> io::Result<usize> {
        check_for_broken_pipe(self.0.write_vectored(bufs))
    }
}

fn check_for_broken_pipe<T>(result: io::Result<T>) -> io::Result<T> {
    match result {
        Err(ref err) if err.kind() == io::ErrorKind::BrokenPipe => exit_for_broken_pipe(),
        result => result,
    }
}

fn exit_for_broken_pipe() -> ! {
    #[cfg(unix)]
    try_terminating_by_sigpipe();

    // Outside of Unix, or in other cases where dying from SIGPIPE fails,
    // we fall back to a plain exit with a generic code.
    std::process::exit(1);
}

#[cfg(unix)]
fn try_terminating_by_sigpipe() {
    // TODO: This needs to be replaced with sigaction as mentioned in the README.
    unsafe { libc::signal(libc::SIGPIPE, libc::SIG_DFL) };

    // SAFETY: POSIX.1-2001 requires raise to be async-signal-safe (according to glibc manpages),
    // so we ought to be safe from data races. It's possible that a thread raced with us above to
    // set a non-default handler for SIGPIPE, in which case we'll invoke that handler before
    // continuing with a plain exit.
    //
    // TODO: The race condition described above needs to be documented at the crate level.
    // However, signal handler registration generally involves unsafe code, and the most important
    // safe crate for signal handling (signal_hook) describes this race in its own documentation as
    // well (at least in signal_hook::low_level::emulate_default_handler).
    //
    // (I'm not using signal_hook directly since the emulation falls back to abort instead of an
    // exit, which adds another layer of signal handling messiness and maybe other undesirable side
    // effects.)
    unsafe { libc::raise(libc::SIGPIPE) };

    // That should have been synchronous. If we get here, it could be that the
    // signal was blocked, or that another thread raced to install a handler,
    // or that a libc call somehow failed and we ignored it...
    // regardless, we'll fall back to the plain exit above.
}
