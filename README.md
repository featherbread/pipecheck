`pipecheck` is a Rust `Write` wrapper that handles broken pipe errors by terminating the current process.
See [the crate documentation](https://docs.rs/pipecheck/latest/pipecheck/) for details.

## WARNINGS AND DEFICIENCIES

On Unix, this crate resets (using `unsafe` code) SIGPIPE to its default behavior
during program termination. If the installation of a non-default handler for
SIGPIPE races with this reset, this crate may invoke the newly installed handler
before terminating the program with exit code 1. This is not expected to invoke
data races or other Undefined Behavior, as POSIX.1 requires the two involved
system calls (`signal` and `raise`) to be async-signal-safe. However, the crate
documentation fails to discuss this behavior.

The use of `signal` rather than `sigaction` should be considered a deficiency
in the current implementation, as the semantics of the former are less consistent
across implementations and some systems document the effects of `signal` in a
multi-threaded process to be unspecified. (That said, `SIG_DFL` is a less
dangerous case than setting a custom handler with `signal`.)

The current implementation does not modify signal masks to unblock SIGPIPE.
If SIGPIPE is blocked, it falls back to exiting with code 1. This behavior may
or may not change in the future. The Go implementation referenced in the crate
documentation _does_ unblock SIGPIPE, but blocking SIGPIPE in the first place
in a Rust program seems odd when the default behavior is already to ignore it.

You are strongly encouraged to judge the author's understanding of Unix and signals
on the basis of these implementation deficiencies when deciding whether
to adopt this crate for long-term use, especially given the author's choice
to introduce this crate into the broader Rust ecosystem with these deficiencies intact.
Note that even this list of deficiencies is based on the author's own self-review.

It is expected that work will be done to correct these deficiencies over time,
and old versions of the crate may be yanked regardless of how likely the
deficiencies are to cause issues for real-world programs.

## History

`pipecheck` was first implemented as a private module in [xt](https://github.com/featherbread/xt)
in May 2023, then extracted into an independent crate in October 2025.
