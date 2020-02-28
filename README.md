Jane-Eyre
=========

[![Build Status][actions-badge]][actions-url]
[![Latest Version](https://img.shields.io/crates/v/eyre.svg)](https://crates.io/crates/eyre)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/eyre)

This library provides [`eyre::ErrReport`][ErrReport], a trait object based
error handling type for easy idiomatic error handling and reporting in Rust
applications.

This crate is a fork of `anyhow` by @dtolnay. My goal in writing this crate is
to explore new ways to associate context with errors, to cleanly separate the
concept of an error and context about an error, and to more clearly communicate
the intended usage of this crate via changes to the API.

The main changes this crate brings to anyhow are

* Addition of the [`eyre::EyreContext`] trait and a type parameter on the core error
  handling type which users can use to insert custom forms of context into
  their catch all error handling type.
* Rebranding the type as principally for error reporting, rather than
  describing it as an error type in its own right. This type is not an error,
  it contains errors that it masqerades as, and provides helpers for creating
  new errors to wrap those errors and for displaying those chains of errors,
  and the included context, to the end user. The goal is to make it obvious
  that this type is meant to be used when the only way you expect to handle
  errors is to print them.
* Changing the [`anyhow::Context`] trait to [`eyre::WrapErr`] to make it clear
  that it is unrelated to the [`eyre::EyreContext`] and the context member, and
  is only for inserting new errors into the chain of errors.
* Addition of a new `context` function on [`eyre::ErrReport`] to assist with
  extracting members from the inner Context, which is used by
  [`eyre::ErrReport`] to extract [`std::backtrace::Backtrace`]'s from generic
  contexts types.

These changes were made in order to facilitate the usage of
[`tracing::SpanTrace`] with anyhow, which is a Backtrace-like type for
rendering custom defined runtime context.

**Note**: The way the `eyre!` macro works in practice differs from how
`anyhow!` works due to the addition of the generic type parameter. In anyhow
the following is valid.

```rust
// Works
let val = get_optional_val.ok_or_else(|| anyhow!("failed to get value)).unwrap();
```

Where as with `eyre!` this will fail due to being unable to infer the type for
the Context parameter. The solution to this problem, should you encounter it,
is to give the compiler a hint for what type it should be resolving to, either
via your return type or a type annotation.

```rust
// Broken
let val = get_optional_val.ok_or_else(|| eyre!("failed to get value)).unwrap();

// Works
let val: ErrReport = get_optional_val.ok_or_else(|| eyre!("failed to get value)).unwrap();
```
[ErrReport]: https://docs.rs/eyre/1.0/eyre/struct.ErrReport.html
[actions-badge]: https://github.com/yaahc/eyre/workflows/Continuous%20integration/badge.svg
[actions-url]: https://github.com/yaahc/eyre/actions?query=workflow%3A%22Continuous+integration%22

## Customization

In order to insert your own custom context type you must first implement the
`eyre::EyreContext` trait for said type, which has four required methods.

* `fn default(error: &Error) -> Self` - For constructing default context while
allowing special case handling depending on the content of the error you're
wrapping.

This is essentially `Default::default` but more flexible, for example, the
`eyre::DefaultContext` type provide by this crate uses this to only capture a
`Backtrace` if the inner `Error` does not already have one.

```rust
fn default(error: &(dyn StdError + 'static)) -> Self {
    let backtrace = backtrace_if_absent!(error);

    Self { backtrace }
}
```

* `fn context_raw(&self, typeid TypeID) -> Option<&dyn Any>` - For extracting
  arbitrary members from a context based on their type.

This method is like a flexible version of the `fn backtrace(&self)` method on
the `Error` trait. In the future we will likely support extracting `Backtrace`s
and `SpanTrace`s by default by relying on the implementation of `context_raw`
provided by the user.


Here is how the `eyre::DefaultContext` type uses this to return `Backtrace`s.

```rust
fn context_raw(&self, typeid: TypeId) -> Option<&dyn Any> {
    if typeid == TypeId::of::<Backtrace>() {
        self.backtrace.as_ref().map(|b| b as &dyn Any)
    } else {
        None
    }
}
```

* `fn debug(&self, error: &(dyn Error + 'static), f: &mut fmt::Formatter<'_>) -> fmt Result`
  it's companion `display` version. - For formatting the entire error chain and
  the user provided context.

When overriding the context it no longer makes sense for `eyre::ErrReport` to
provide the `Display` and `Debug` implementations for the user, becase we
cannot predict what forms of context you will need to display along side your
chain of errors. Instead we forward the implementations of `Display` and
`Debug` to these methods on the inner `EyreContext` type.

This crate does provide a few helpers to assist in writing display
implementations, specifically the `Chain` type, for treating an error and its
sources like an iterator, and the `Indented` type, for indenting multi line
errors consistently without using heap allocations.

**Note**: best practices for printing errors suggest that `{}` should only
print the current error and none of its sources, and that the primary method of
displaying an error, its sources, and its context should be handled by the
`Debug` implementation, which is what is used to print errors that are returned
from `main`. For examples on how to implement this please refer to the
implementations of `display` and `debug` on `eyre::DefaultContext`

Once you've defined a custom Context type you can use it throughout your
application by defining a type alias.

```rust
type ErrReport = eyre::ErrReport<MyContext>;

// And optionally...
type Result<T, E = eyre::ErrReport<MyContext>> = core::result::Result<T, E>;
```

```toml
[dependencies]
eyre = "0.2"
```

<br>

## Details

- Use `Result<T, eyre::ErrReport>`, or equivalently `eyre::Result<T>`, as the
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

- Attach context to help the person troubleshooting the error understand where
  things went wrong. A low-level error like "No such file or directory" can be
  annoying to debug without more context about what higher level step the
  application was in the middle of.

  ```rust
  use eyre::{WrapErr, Result};

  fn main() -> Result<()> {
      ...
      it.detach().context("Failed to detach the important thing")?;

      let content = std::fs::read(path)
          .with_context(|| format!("Failed to read instrs from {}", path))?;
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

- A backtrace is captured and printed with the error if the underlying error
  type does not already provide its own. In order to see backtraces, the
  `RUST_LIB_BACKTRACE=1` environment variable must be defined.

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
  supports string interpolation and produces an `eyre::ErrReport`.

  ```rust
  return Err(eyre!("Missing attribute: {}", missing));
  ```

<br>

## No-std support

**NOTE**: tests are currently broken for `no_std` so I cannot guaruntee that
everything works still. I'm waiting for upstream fixes to be merged rather than
fixing them myself, so bear with me.

In no_std mode, the same API is almost all available and works the same way. To
depend on Eyre in no_std mode, disable our default enabled "std" feature in
Cargo.toml. A global allocator is required.

```toml
[dependencies]
eyre = { version = "0.2", default-features = false }
```

Since the `?`-based error conversions would normally rely on the
`std::error::Error` trait which is only available through std, no_std mode will
require an explicit `.map_err(ErrReport::msg)` when working with a non-Eyre error
type inside a function that returns Eyre's error type.

<br>

## Comparison to failure

The `eyre::ErrReport` type works something like `failure::Error`, but unlike
failure ours is built around the standard library's `std::error::Error` trait
rather than a separate trait `failure::Fail`. The standard library has adopted
the necessary improvements for this to be possible as part of [RFC 2504].

[RFC 2504]: https://github.com/rust-lang/rfcs/blob/master/text/2504-fix-error.md

<br>

## Comparison to thiserror

Use Eyre if you don't care what error type your functions return, you just
want it to be easy. This is common in application code. Use [thiserror] if you
are a library that wants to design your own dedicated error type(s) so that on
failures the caller gets exactly the information that you choose.

[thiserror]: https://github.com/dtolnay/thiserror

<br>

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
