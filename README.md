eyre
====

[![Build Status][actions-badge]][actions-url]
[![Latest Version](https://img.shields.io/crates/v/eyre.svg)](https://crates.io/crates/eyre)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/eyre)

This library provides [`eyre::Report`][Report], a trait object based
error handling type for easy idiomatic error handling and reporting in Rust
applications.

This crate is a fork of [`anyhow`] by @dtolnay with a support for customized
`Reports`. For more details on customization checkout the docs on
[`eyre::EyreContext`].

## Custom Contexts

The heart of this crate is its ability to swap out the Context type to change
what information is carried alongside errors and how the end report is
formatted. This crate is meant to be used alongside companion crates that
customize its behavior. Below is a list of known custom context crates and
short summaries of what features they provide.

- [`stable-eyre`]: Switches the backtrace type from `std`'s to `backtrace-rs`'s
  so that it can be captured on stable. The report format is identical to
  `DefaultContext`'s report format.
- [`color-eyre`]: Captures a `backtrace::Backtrace` and a
  `tracing_error::SpanTrace`. Provides a `Help` trait for attaching warnings
  and suggestions to error reports. The end report is then pretty printed with
  the help of [`color-backtrace`], [`color-spantrace`], and `ansi_term`. Check
  out the README on [`color-eyre`] for screenshots of the report format.
- [`simple-eyre`]: A minimal `EyreContext` that captures no additional
  information, for when you do not wish to capture `Backtrace`s with errors.
- [`jane-eyre`]: A custom context type that exists purely for the pun.
  Currently just re-exports `color-eyre`.

## Details

- Use `Result<T, eyre::Report>`, or equivalently `eyre::Result<T>`, as the
  return type of any fallible function.

  Within the function, use `?` to easily propagate any error that implements the
  `std::error::Error` trait.

  ```rust
  use eyre::Result;

  fn get_cluster_info() -> Result<ClusterMap> {
      let config = std::fs::read_to_string("cluster.json")?;
      let map: ClusterMap = serde_json::from_str(&config)?;
      Ok(map)
  }
  ```

- Wrap a lower level error with a new error created from a message to help the
  person troubleshooting understand what the chain of failures that occured. A
  low-level error like "No such file or directory" can be annoying to debug
  without more information about what higher level step the application was in
  the middle of.

  ```rust
  use eyre::{WrapErr, Result};

  fn main() -> Result<()> {
      ...
      it.detach().wrap_err("Failed to detach the important thing")?;

      let content = std::fs::read(path)
          .wrap_err_with(|| format!("Failed to read instrs from {}", path))?;
      ...
  }
  ```

  ```console
  Error: Failed to read instrs from ./path/to/instrs.json

  Caused by:
      No such file or directory (os error 2)
  ```

- Downcasting is supported and can be by value, by shared reference, or by
  mutable reference as needed.

  ```rust
  // If the error was caused by redaction, then return a
  // tombstone instead of the content.
  match root_cause.downcast_ref::<DataStoreError>() {
      Some(DataStoreError::Censored(_)) => Ok(Poll::Ready(REDACTED_CONTENT)),
      None => Err(error),
  }
  ```

- If using the nightly channel, a backtrace is captured and printed with the
  error if the underlying error type does not already provide its own. In order
  to see backtraces, they must be enabled through the environment variables
  described in [`std::backtrace`]:

  - If you want panics and errors to both have backtraces, set
    `RUST_BACKTRACE=1`;
  - If you want only errors to have backtraces, set `RUST_LIB_BACKTRACE=1`;
  - If you want only panics to have backtraces, set `RUST_BACKTRACE=1` and
    `RUST_LIB_BACKTRACE=0`.

  The tracking issue for this feature is [rust-lang/rust#53487].

  [`std::backtrace`]: https://doc.rust-lang.org/std/backtrace/index.html#environment-variables
  [rust-lang/rust#53487]: https://github.com/rust-lang/rust/issues/53487

- Eyre works with any error type that has an impl of `std::error::Error`,
  including ones defined in your crate. We do not bundle a `derive(Error)` macro
  but you can write the impls yourself or use a standalone macro like
  [thiserror].

  ```rust
  use thiserror::Error;

  #[derive(Error, Debug)]
  pub enum FormatError {
      #[error("Invalid header (expected {expected:?}, got {found:?})")]
      InvalidHeader {
          expected: String,
          found: String,
      },
      #[error("Missing attribute: {0}")]
      MissingAttribute(String),
  }
  ```

- One-off error messages can be constructed using the `eyre!` macro, which
  supports string interpolation and produces an `eyre::Report`.

  ```rust
  return Err(eyre!("Missing attribute: {}", missing));
  ```

## No-std support

**NOTE**: tests are currently broken for `no_std` so I cannot guaruntee that
everything works still. I'm waiting for upstream fixes to be merged rather than
fixing them myself, so bear with me.

In no_std mode, the same API is almost all available and works the same way. To
depend on Eyre in no_std mode, disable our default enabled "std" feature in
Cargo.toml. A global allocator is required.

```toml
[dependencies]
eyre = { version = "0.4", default-features = false }
```

Since the `?`-based error conversions would normally rely on the
`std::error::Error` trait which is only available through std, no_std mode will
require an explicit `.map_err(Report::msg)` when working with a non-Eyre error
type inside a function that returns Eyre's error type.

## Comparison to failure

The `eyre::Report` type works something like `failure::Error`, but unlike
failure ours is built around the standard library's `std::error::Error` trait
rather than a separate trait `failure::Fail`. The standard library has adopted
the necessary improvements for this to be possible as part of [RFC 2504].

[RFC 2504]: https://github.com/rust-lang/rfcs/blob/master/text/2504-fix-error.md

## Comparison to thiserror

Use Eyre if you don't care what error type your functions return, you just
want it to be easy. This is common in application code. Use [thiserror] if you
are a library that wants to design your own dedicated error type(s) so that on
failures the caller gets exactly the information that you choose.

[thiserror]: https://github.com/dtolnay/thiserror

## Compatibility with `anyhow`

This crate does its best to be usable as a drop in replacement of `anyhow` and
vice-versa by `re-exporting` all of the renamed APIs with the names used in
`anyhow`.

There are two main incompatibilities that you might encounter when porting a
codebase from `anyhow` to `eyre`:

- type inference errors when using `eyre!`
- `.context` not being implemented for `Option`

#### Type Inference Errors

The type inference issue is caused by the generic parameter, which isn't
present in `anyhow::Error`. Specifically, the following works in anyhow:

```rust
use anyhow::anyhow;

// Works
let val = get_optional_val().ok_or_else(|| anyhow!("failed to get value")).unwrap_err();
```

Where as with `eyre!` this will fail due to being unable to infer the type for
the Context parameter. The solution to this problem, should you encounter it,
is to give the compiler a hint for what type it should be resolving to, either
via your return type or a type annotation.

```rust,compile_fail
use eyre::eyre;

// Broken
let val = get_optional_val().ok_or_else(|| eyre!("failed to get value")).unwrap();

// Works
let val: Report = get_optional_val().ok_or_else(|| eyre!("failed to get value")).unwrap();
```

#### `Context` and `Option`

As part of renaming `Context` to `WrapErr` we also intentionally do not
implement `WrapErr` for `Option`. This decision was made because `wrap_err`
implies that you're creating a new error that saves the old error as its
`source`. With `Option` there is no source error to wrap, so `wrap_err` ends up
being somewhat meaningless.

Instead `eyre` intends for users to use the combinator functions provided by
`std` for converting `Option`s to `Result`s. So where you would write this with
anyhow:

```rust
use anyhow::Context;

let opt: Option<()> = None;
let result = opt.context("new error message");
```

With `eyre` we want users to write:

```rust
use eyre::{eyre, Result};

let opt: Option<()> = None;
let result: Result<()> = opt.ok_or_else(|| eyre!("new error message"));
```

However, to help with porting we do provide a `ContextCompat` trait which
implements `context` for options which you can import to make existing
`.context` calls compile.

[Report]: https://docs.rs/eyre/*/eyre/struct.Report.html
[`eyre::EyreContext`]: https://docs.rs/eyre/*/eyre/trait.EyreContext.html
[`eyre::WrapErr`]: https://docs.rs/eyre/*/eyre/trait.WrapErr.html
[`anyhow::Context`]: https://docs.rs/anyhow/*/anyhow/trait.Context.html
[`anyhow`]: https://github.com/dtolnay/anyhow
[`tracing_error::SpanTrace`]: https://docs.rs/tracing-error/*/tracing_error/struct.SpanTrace.html
[`stable-eyre`]: https://github.com/yaahc/stable-eyre
[`color-eyre`]: https://github.com/yaahc/color-eyre
[`jane-eyre`]: https://github.com/yaahc/jane-eyre
[`simple-eyre`]: https://github.com/yaahc/simple-eyre
[`color-spantrace`]: https://github.com/yaahc/color-spantrace
[`color-backtrace`]: https://github.com/athre0z/color-backtrace
[actions-badge]: https://github.com/yaahc/eyre/workflows/Continuous%20integration/badge.svg
[actions-url]: https://github.com/yaahc/eyre/actions?query=workflow%3A%22Continuous+integration%22


#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
