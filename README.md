color-eyre
----------

A custom context for the [`eyre`] crate for colorful error reports, suggestions,
and [`tracing-error`] support.

**Disclaimer**: This crate is currently pre-release while I try to upstream
changes I made to [`color-backtrace`] to support this crate. Until then I
cannot publish this to crates.io, the documentation is filled out however so
simply run `cargo doc --open` for an explanation of usage.

## Explanation

This crate works by defining a `Context` type which implements `EyreContext`
and a pair of type aliases for setting this context object as the parameter of
`eyre::Report`.

```rust
pub type Report = eyre::Report<Context>;
pub type Result<T, E = Report> = core::result::Result<T, E>;
```

## Features

- captures a [`backtrace::Backtrace`] and prints using [`color-backtrace`]
- captures a [`tracing_error::SpanTrace`] and prints using
[`color-spantrace`]
- Only capture SpanTrace by default for better performance.
- display source lines when `RUST_LIB_BACKTRACE=full` is set from both of
  the above libraries
- store help text via [`Help`] trait and display after final report

## Setup

Add the following to your toml file:

```toml
[dependencies]
eyre = "0.3.8"
color-eyre = "0.2.0"
```

And then import the type alias from color-eyre for `Report` or `Result`.

```rust
use color_eyre::Report;

// or

fn example(&self) -> color_eyre::Result<()> {
    // ...
}
```

## Minimal Report Format

![minimal report format](./pictures/minimal.png)

## Short Report Format (with `RUST_LIB_BACKTRACE=1`)

![short report format](./pictures/short.png)

## Full Report Format (with `RUST_LIB_BACKTRACE=full`)

![full report format](./pictures/full.png)

[`eyre`]: https://docs.rs/eyre
[`tracing-error`]: https://docs.rs/tracing-error
[`color-backtrace`]: https://docs.rs/color-backtrace
[`eyre::EyreContext`]: https://docs.rs/eyre/0.3.8/eyre/trait.EyreContext.html
[`backtrace::Backtrace`]: https://docs.rs/backtrace/0.3.46/backtrace/struct.Backtrace.html
[`tracing_error::SpanTrace`]: https://docs.rs/tracing-error/0.1.2/tracing_error/struct.SpanTrace.html
[`color-spantrace`]: https://github.com/yaahc/color-spantrace
[`Help`]: trait.Help.html
[`eyre::Report`]: https://docs.rs/eyre/0.3.8/eyre/struct.Report.html
