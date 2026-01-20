`pipecheck` is a Rust `Write` wrapper that handles broken pipe errors by terminating the current process.
See [the crate documentation](https://docs.rs/pipecheck/latest/pipecheck/) for details.

## WARNINGS AND DEFICIENCIES

The use of `signal` rather than `sigaction` should be considered a deficiency
in the current implementation, as the semantics of the former are less consistent
across implementations and some systems document the effects of `signal` in a
multi-threaded process to be unspecified. Work is in progress to migrate to
sigaction; the new implementation is yet to be properly tested.

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
