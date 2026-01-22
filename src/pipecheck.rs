//! Cross-platform Unix-style handling of broken pipe errors.
//!
//! This module is sourced from <https://crates.io/crates/pipecheck>.
//! See the documentation of that crate for notable behavioral caveats
//! and background information on SIGPIPE handling in Rust.
//!
//! # MIT License
//!
//! Copyright (c) 2025 Alex Hamlin
//!
//! Permission is hereby granted, free of charge, to any person obtaining a copy
//! of this software and associated documentation files (the "Software"), to deal
//! in the Software without restriction, including without limitation the rights
//! to use, copy, modify, merge, publish, distribute, sublicense, and/or sell
//! copies of the Software, and to permit persons to whom the Software is
//! furnished to do so, subject to the following conditions:
//!
//! The above copyright notice and this permission notice shall be included in all
//! copies or substantial portions of the Software.
//!
//! THE SOFTWARE IS PROVIDED "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR
//! IMPLIED, INCLUDING BUT NOT LIMITED TO THE WARRANTIES OF MERCHANTABILITY,
//! FITNESS FOR A PARTICULAR PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE
//! AUTHORS OR COPYRIGHT HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER
//! LIABILITY, WHETHER IN AN ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM,
//! OUT OF OR IN CONNECTION WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE
//! SOFTWARE.

use std::io::{self, Write};

/// A convenient alias for [`Writer::new`].
pub fn wrap<W: Write>(w: W) -> Writer<W> {
    Writer::new(w)
}

/// A writer that silently terminates the program on broken pipe errors.
///
/// When any call to its underlying writer returns a [`BrokenPipe`](io::ErrorKind::BrokenPipe)
/// error, a `Writer` terminates the current process with a SIGPIPE signal, or falls back to a
/// plain exit with code 1.
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
    // we fall back to a plain exit with the most generic code.
    std::process::exit(1);
}

#[cfg(unix)]
fn try_terminating_by_sigpipe() {
    use std::mem::MaybeUninit;
    use std::ptr;

    // SAFETY: sigaction is a C struct, so zeroed() is a valid type-level initialization.
    // Rust's usual struct initializer syntax is a bad idea,
    // since certain platforms might have extra fields we aren't ready for.
    let mut act: libc::sigaction = unsafe { MaybeUninit::zeroed().assume_init() };
    act.sa_sigaction = libc::SIG_DFL;

    // SAFETY: All the values should be valid; the last argument in particular
    // is explicitly allowed to be null if we don't care about the previous handler.
    // POSIX.1 requires this to be reentrant in multi-threaded programs.
    let ret = unsafe { libc::sigaction(libc::SIGPIPE, &act, ptr::null_mut()) };
    if ret != 0 {
        return; // Weird, sigaction failed. Best we can do is fall back to a plain exit.
    }

    // SAFETY: We know SIGPIPE is a valid signal value, and POSIX.1 requires this
    // to be reentrant in multi-threaded programs. This _should_ terminate the program,
    // but might not due to behavioral caveats documented in the upstream pipecheck crate.
    unsafe { libc::raise(libc::SIGPIPE) };

    // If that failed to terminate the process, we fall through this function and exit.
}
