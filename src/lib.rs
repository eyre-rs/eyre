//! This library provides [`eyre::Report`][Report], a trait object based
//! error handling type for easy idiomatic error handling and reporting in Rust
//! applications.
//!
//! This crate is a fork of [`anyhow`] by @dtolnay with a support for customized
//! `Reports`. For more details on customization checkout the docs on
//! [`eyre::EyreHandler`].
//!
//! ## Custom Report Handlers
//!
//! The heart of this crate is its ability to swap out the Handler type to change
//! what information is carried alongside errors and how the end report is
//! formatted. This crate is meant to be used alongside companion crates that
//! customize its behavior. Below is a list of known crates that export report
//! handlers for eyre and short summaries of what features they provide.
//!
//! - [`stable-eyre`]: Switches the backtrace type from `std`'s to `backtrace-rs`'s
//!   so that it can be captured on stable. The report format is identical to
//!   `DefaultHandler`'s report format.
//! - [`color-eyre`]: Captures a `backtrace::Backtrace` and a
//!   `tracing_error::SpanTrace`. Provides a `Help` trait for attaching warnings
//!   and suggestions to error reports. The end report is then pretty printed with
//!   the help of [`color-backtrace`], [`color-spantrace`], and `ansi_term`. Check
//!   out the README on [`color-eyre`] for screenshots of the report format.
//! - [`simple-eyre`]: A minimal `EyreHandler` that captures no additional
//!   information, for when you do not wish to capture `Backtrace`s with errors.
//! - [`jane-eyre`]: A a report handler crate that exists purely for the pun.
//! Currently just re-exports `color-eyre`.
//!
//! ## Details
//!
//! - Use `Result<T, eyre::Report>`, or equivalently `eyre::Result<T>`, as the
//!   return type of any fallible function.
//!
//!   Within the function, use `?` to easily propagate any error that implements the
//!   `std::error::Error` trait.
//!
//!   ```rust
//!   # pub trait Deserialize {}
//!   #
//!   # mod serde_json {
//!   #     use super::Deserialize;
//!   #     use std::io;
//!   #
//!   #     pub fn from_str<T: Deserialize>(json: &str) -> io::Result<T> {
//!   #         unimplemented!()
//!   #     }
//!   # }
//!   #
//!   # struct ClusterMap;
//!   #
//!   # impl Deserialize for ClusterMap {}
//!   #
//!   use eyre::Result;
//!
//!   fn get_cluster_info() -> Result<ClusterMap> {
//!       let config = std::fs::read_to_string("cluster.json")?;
//!       let map: ClusterMap = serde_json::from_str(&config)?;
//!       Ok(map)
//!   }
//!   #
//!   # fn main() {}
//!   ```
//!
//! - Wrap a lower level error with a new error created from a message to help the
//!   person troubleshooting understand what the chain of failures that occured. A
//!   low-level error like "No such file or directory" can be annoying to debug
//!   without more information about what higher level step the application was in
//!   the middle of.
//!
//!   ```rust
//!   # struct It;
//!   #
//!   # impl It {
//!   #     fn detach(&self) -> Result<()> {
//!   #         unimplemented!()
//!   #     }
//!   # }
//!   #
//!   use eyre::{WrapErr, Result};
//!
//!   fn main() -> Result<()> {
//!       # return Ok(());
//!       #
//!       # const _: &str = stringify! {
//!       ...
//!       # };
//!       #
//!       # let it = It;
//!       # let path = "./path/to/instrs.json";
//!       #
//!       it.detach().wrap_err("Failed to detach the important thing")?;
//!
//!       let content = std::fs::read(path)
//!           .wrap_err_with(|| format!("Failed to read instrs from {}", path))?;
//!       #
//!       # const _: &str = stringify! {
//!       ...
//!       # };
//!       #
//!       # Ok(())
//!   }
//!   ```
//!
//!   ```console
//!   Error: Failed to read instrs from ./path/to/instrs.json
//!
//!   Caused by:
//!       No such file or directory (os error 2)
//!   ```
//!
//! - Downcasting is supported and can be by value, by shared reference, or by
//!   mutable reference as needed.
//!
//!   ```rust
//!   # use eyre::{Report, eyre};
//!   # use std::fmt::{self, Display};
//!   # use std::task::Poll;
//!   #
//!   # #[derive(Debug)]
//!   # enum DataStoreError {
//!   #     Censored(()),
//!   # }
//!   #
//!   # impl Display for DataStoreError {
//!   #     fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
//!   #         unimplemented!()
//!   #     }
//!   # }
//!   #
//!   # impl std::error::Error for DataStoreError {}
//!   #
//!   # const REDACTED_CONTENT: () = ();
//!   #
//!   # let error: Report = eyre!("...");
//!   # let root_cause = &error;
//!   #
//!   # let ret =
//!   // If the error was caused by redaction, then return a
//!   // tombstone instead of the content.
//!   match root_cause.downcast_ref::<DataStoreError>() {
//!       Some(DataStoreError::Censored(_)) => Ok(Poll::Ready(REDACTED_CONTENT)),
//!       None => Err(error),
//!   }
//!   # ;
//!   ```
//!
//! - If using the nightly channel, a backtrace is captured and printed with the
//!   error if the underlying error type does not already provide its own. In order
//!   to see backtraces, they must be enabled through the environment variables
//!   described in [`std::backtrace`]:
//!
//!   - If you want panics and errors to both have backtraces, set
//!     `RUST_BACKTRACE=1`;
//!   - If you want only errors to have backtraces, set `RUST_LIB_BACKTRACE=1`;
//!   - If you want only panics to have backtraces, set `RUST_BACKTRACE=1` and
//!     `RUST_LIB_BACKTRACE=0`.
//!
//!   The tracking issue for this feature is [rust-lang/rust#53487].
//!
//!   [`std::backtrace`]: https://doc.rust-lang.org/std/backtrace/index.html#environment-variables
//!   [rust-lang/rust#53487]: https://github.com/rust-lang/rust/issues/53487
//!
//! - Eyre works with any error type that has an impl of `std::error::Error`,
//!   including ones defined in your crate. We do not bundle a `derive(Error)` macro
//!   but you can write the impls yourself or use a standalone macro like
//!   [thiserror].
//!
//!   ```rust
//!   use thiserror::Error;
//!
//!   #[derive(Error, Debug)]
//!   pub enum FormatError {
//!       #[error("Invalid header (expected {expected:?}, got {found:?})")]
//!       InvalidHeader {
//!           expected: String,
//!           found: String,
//!       },
//!       #[error("Missing attribute: {0}")]
//!       MissingAttribute(String),
//!   }
//!   ```
//!
//! - One-off error messages can be constructed using the `eyre!` macro, which
//!   supports string interpolation and produces an `eyre::Report`.
//!
//!   ```rust
//!   # use eyre::{eyre, Result};
//!   #
//!   # fn demo() -> Result<()> {
//!   #     let missing = "...";
//!   return Err(eyre!("Missing attribute: {}", missing));
//!   #     Ok(())
//!   # }
//!   ```
//!
//! ## No-std support
//!
//! **NOTE**: tests are currently broken for `no_std` so I cannot guarantee that
//! everything works still. I'm waiting for upstream fixes to be merged rather than
//! fixing them myself, so bear with me.
//!
//! In no_std mode, the same API is almost all available and works the same way. To
//! depend on Eyre in no_std mode, disable our default enabled "std" feature in
//! Cargo.toml. A global allocator is required.
//!
//! ```toml
//! [dependencies]
//! eyre = { version = "0.6", default-features = false }
//! ```
//!
//! Since the `?`-based error conversions would normally rely on the
//! `std::error::Error` trait which is only available through std, no_std mode will
//! require an explicit `.map_err(Report::msg)` when working with a non-Eyre error
//! type inside a function that returns Eyre's error type.
//!
//! ## Comparison to failure
//!
//! The `eyre::Report` type works something like `failure::Error`, but unlike
//! failure ours is built around the standard library's `std::error::Error` trait
//! rather than a separate trait `failure::Fail`. The standard library has adopted
//! the necessary improvements for this to be possible as part of [RFC 2504].
//!
//! [RFC 2504]: https://github.com/rust-lang/rfcs/blob/master/text/2504-fix-error.md
//!
//! ## Comparison to thiserror
//!
//! Use Eyre if you don't care what error type your functions return, you just
//! want it to be easy. This is common in application code. Use [thiserror] if you
//! are a library that wants to design your own dedicated error type(s) so that on
//! failures the caller gets exactly the information that you choose.
//!
//! [thiserror]: https://github.com/dtolnay/thiserror
//!
//! ## Compatibility with `anyhow`
//!
//! This crate does its best to be usable as a drop in replacement of `anyhow` and
//! vice-versa by `re-exporting` all of the renamed APIs with the names used in
//! `anyhow`.
//!
//! There are two main incompatibilities that you might encounter when porting a
//! codebase from `anyhow` to `eyre`:
//!
//! - type inference errors when using `eyre!`
//! - `.context` not being implemented for `Option`
//!
//! #### Type Inference Errors
//!
//! The type inference issue is caused by the generic parameter, which isn't
//! present in `anyhow::Error`. Specifically, the following works in anyhow:
//!
//! ```rust
//! # fn get_optional_val() -> Option<()> { None };
//! use anyhow::anyhow;
//!
//! // Works
//! let val = get_optional_val().ok_or_else(|| anyhow!("failed to get value")).unwrap_err();
//! ```
//!
//! Where as with `eyre!` this will fail due to being unable to infer the type for
//! the Handler parameter. The solution to this problem, should you encounter it,
//! is to give the compiler a hint for what type it should be resolving to, either
//! via your return type or a type annotation.
//!
//! ```rust,compile_fail
//! use eyre::eyre;
//!
//! # fn get_optional_val() -> Option<()> { None };
//! // Broken
//! let val = get_optional_val().ok_or_else(|| eyre!("failed to get value")).unwrap();
//!
//! // Works
//! let val: Report = get_optional_val().ok_or_else(|| eyre!("failed to get value")).unwrap();
//! ```
//!
//! #### `Context` and `Option`
//!
//! As part of renaming `Context` to `WrapErr` we also intentionally do not
//! implement `WrapErr` for `Option`. This decision was made because `wrap_err`
//! implies that you're creating a new error that saves the old error as its
//! `source`. With `Option` there is no source error to wrap, so `wrap_err` ends up
//! being somewhat meaningless.
//!
//! Instead `eyre` intends for users to use the combinator functions provided by
//! `std` for converting `Option`s to `Result`s. So where you would write this with
//! anyhow:
//!
//! ```rust
//! use anyhow::Context;
//!
//! let opt: Option<()> = None;
//! let result = opt.context("new error message");
//! ```
//!
//! With `eyre` we want users to write:
//!
//! ```rust
//! use eyre::{eyre, Result};
//!
//! let opt: Option<()> = None;
//! let result: Result<()> = opt.ok_or_else(|| eyre!("new error message"));
//! ```
//!
//! However, to help with porting we do provide a `ContextCompat` trait which
//! implements `context` for options which you can import to make existing
//! `.context` calls compile.
//!
//! [Report]: https://docs.rs/eyre/*/eyre/struct.Report.html
//! [`eyre::EyreHandler`]: https://docs.rs/eyre/*/eyre/trait.EyreHandler.html
//! [`eyre::WrapErr`]: https://docs.rs/eyre/*/eyre/trait.WrapErr.html
//! [`anyhow::Context`]: https://docs.rs/anyhow/*/anyhow/trait.Context.html
//! [`anyhow`]: https://github.com/dtolnay/anyhow
//! [`tracing_error::SpanTrace`]: https://docs.rs/tracing-error/*/tracing_error/struct.SpanTrace.html
//! [`stable-eyre`]: https://github.com/yaahc/stable-eyre
//! [`color-eyre`]: https://github.com/yaahc/color-eyre
//! [`jane-eyre`]: https://github.com/yaahc/jane-eyre
//! [`simple-eyre`]: https://github.com/yaahc/simple-eyre
//! [`color-spantrace`]: https://github.com/yaahc/color-spantrace
//! [`color-backtrace`]: https://github.com/athre0z/color-backtrace
#![doc(html_root_url = "https://docs.rs/eyre/0.6.0")]
#![warn(
    missing_debug_implementations,
    missing_docs,
    missing_doc_code_examples,
    rust_2018_idioms,
    unreachable_pub,
    bad_style,
    const_err,
    dead_code,
    improper_ctypes,
    non_shorthand_field_patterns,
    no_mangle_generic_items,
    overflowing_literals,
    path_statements,
    patterns_in_fns_without_body,
    private_in_public,
    unconditional_recursion,
    unused,
    unused_allocation,
    unused_comparisons,
    unused_parens,
    while_true
)]
#![cfg_attr(backtrace, feature(backtrace))]
#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![allow(
    clippy::needless_doctest_main,
    clippy::new_ret_no_self,
    clippy::wrong_self_convention
)]

#[macro_use]
mod backtrace;
mod chain;
mod context;
mod error;
mod fmt;
mod kind;
mod macros;
mod wrapper;

use crate::backtrace::Backtrace;
use crate::error::ErrorImpl;
use core::fmt::Display;
use core::mem::ManuallyDrop;

use std::error::Error as StdError;

pub use eyre as format_err;
/// Compatibility re-export of `eyre` for interopt with `anyhow`
pub use eyre as anyhow;
use once_cell::sync::OnceCell;
#[doc(hidden)]
pub use DefaultHandler as DefaultContext;
#[doc(hidden)]
pub use EyreHandler as EyreContext;
#[doc(hidden)]
pub use Report as ErrReport;
/// Compatibility re-export of `Report` for interopt with `anyhow`
pub use Report as Error;
/// Compatibility re-export of `WrapErr` for interopt with `anyhow`
pub use WrapErr as Context;

/// The core error reporting type of the library, a wrapper around a dynamic error reporting type.
///
/// `Report` works a lot like `Box<dyn std::error::Error>`, but with these
/// differences:
///
/// - `Report` requires that the error is `Send`, `Sync`, and `'static`.
/// - `Report` guarantees that a backtrace is available, even if the underlying
///   error type does not provide one.
/// - `Report` is represented as a narrow pointer &mdash; exactly one word in
///   size instead of two.
///
/// # Display representations
///
/// When you print an error object using "{}" or to_string(), only the outermost underlying error
/// is printed, not any of the lower level causes. This is exactly as if you had called the Display
/// impl of the error from which you constructed your eyre::Report.
///
/// ```console
/// Failed to read instrs from ./path/to/instrs.json
/// ```
///
/// To print causes as well using eyre's default formatting of causes, use the
/// alternate selector "{:#}".
///
/// ```console
/// Failed to read instrs from ./path/to/instrs.json: No such file or directory (os error 2)
/// ```
///
/// The Debug format "{:?}" includes your backtrace if one was captured. Note
/// that this is the representation you get by default if you return an error
/// from `fn main` instead of printing it explicitly yourself.
///
/// ```console
/// Error: Failed to read instrs from ./path/to/instrs.json
///
/// Caused by:
///     No such file or directory (os error 2)
///
/// Stack backtrace:
///    0: <E as eyre::context::ext::StdError>::ext_report
///              at /git/eyre/src/backtrace.rs:26
///    1: core::result::Result<T,E>::map_err
///              at /git/rustc/src/libcore/result.rs:596
///    2: eyre::context::<impl eyre::WrapErr<T,E,H> for core::result::Result<T,E>>::wrap_err_with
///              at /git/eyre/src/context.rs:58
///    3: testing::main
///              at src/main.rs:5
///    4: std::rt::lang_start
///              at /git/rustc/src/libstd/rt.rs:61
///    5: main
///    6: __libc_start_main
///    7: _start
/// ```
///
/// To see a conventional struct-style Debug representation, use "{:#?}".
///
/// ```console
/// Error {
///     msg: "Failed to read instrs from ./path/to/instrs.json",
///     source: Os {
///         code: 2,
///         kind: NotFound,
///         message: "No such file or directory",
///     },
/// }
/// ```
///
/// If none of the built-in representations are appropriate and you would prefer
/// to render the error and its cause chain yourself, it can be done something
/// like this:
///
/// ```
/// use eyre::{WrapErr, Result};
///
/// fn main() {
///     if let Err(err) = try_main() {
///         eprintln!("ERROR: {}", err);
///         err.chain().skip(1).for_each(|cause| eprintln!("because: {}", cause));
///         std::process::exit(1);
///     }
/// }
///
/// fn try_main() -> Result<()> {
///     # const IGNORE: &str = stringify! {
///     ...
///     # };
///     # Ok(())
/// }
/// ```
pub struct Report {
    inner: ManuallyDrop<Box<ErrorImpl<()>>>,
}

type ErrorHook =
    Box<dyn Fn(&(dyn StdError + 'static)) -> Box<dyn EyreHandler> + Sync + Send + 'static>;

static HOOK: OnceCell<ErrorHook> = OnceCell::new();

/// Error indicating that `set_hook` was unable to install the provided ErrorHook
#[derive(Debug)]
pub struct InstallError;

impl core::fmt::Display for InstallError {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.write_str("cannot install provided ErrorHook, a hook has already been installed")
    }
}

impl StdError for InstallError {}

/// Install the provided error hook for constructing EyreHandlers when converting
/// Errors to Reports
pub fn set_hook(hook: ErrorHook) -> Result<(), InstallError> {
    HOOK.set(hook).map_err(|_| InstallError)
}

fn capture_handler(error: &(dyn StdError + 'static)) -> Box<dyn EyreHandler> {
    let hook = HOOK
        .get_or_init(|| Box::new(DefaultHandler::default_with))
        .as_ref();

    hook(error)
}

impl dyn EyreHandler {
    ///
    pub fn is<T: EyreHandler>(&self) -> bool {
        // Get `TypeId` of the type this function is instantiated with.
        let t = core::any::TypeId::of::<T>();

        // Get `TypeId` of the type in the trait object (`self`).
        let concrete = self.type_id();

        // Compare both `TypeId`s on equality.
        t == concrete
    }

    ///
    pub fn downcast_ref<T: EyreHandler>(&self) -> Option<&T> {
        if self.is::<T>() {
            unsafe { Some(&*(self as *const dyn EyreHandler as *const T)) }
        } else {
            None
        }
    }

    ///
    pub fn downcast_mut<T: EyreHandler>(&mut self) -> Option<&mut T> {
        if self.is::<T>() {
            unsafe { Some(&mut *(self as *mut dyn EyreHandler as *mut T)) }
        } else {
            None
        }
    }
}

/// Error Report Handler trait for customizing `eyre::Report`
pub trait EyreHandler: core::any::Any + Send + Sync {
    /// Define the report format
    ///
    /// Used to override the report format of `eyre::Report`
    ///
    /// # Example
    ///
    /// ```rust
    /// use backtrace::Backtrace;
    /// use eyre::EyreHandler;
    /// use eyre::Chain;
    /// use std::error::Error;
    /// use indenter::indented;
    ///
    /// pub struct Handler {
    ///     backtrace: Backtrace,
    /// }
    ///
    /// impl EyreHandler for Handler {
    ///     fn debug(
    ///         &self,
    ///         error: &(dyn Error + 'static),
    ///         f: &mut core::fmt::Formatter<'_>,
    ///     ) -> core::fmt::Result {
    ///         use core::fmt::Write as _;
    ///
    ///         if f.alternate() {
    ///             return core::fmt::Debug::fmt(error, f);
    ///         }
    ///
    ///         write!(f, "{}", error)?;
    ///
    ///         if let Some(cause) = error.source() {
    ///             write!(f, "\n\nCaused by:")?;
    ///             let multiple = cause.source().is_some();
    ///
    ///             for (n, error) in Chain::new(cause).enumerate() {
    ///                 writeln!(f)?;
    ///                 if multiple {
    ///                     write!(indented(f).ind(n), "{}", error)?;
    ///                 } else {
    ///                     write!(indented(f), "{}", error)?;
    ///                 }
    ///             }
    ///         }
    ///
    ///         let backtrace = &self.backtrace;
    ///         write!(f, "\n\nStack backtrace:\n{:?}", backtrace)?;
    ///
    ///         Ok(())
    ///     }
    /// }
    /// ```
    fn debug(
        &self,
        error: &(dyn StdError + 'static),
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result;

    /// Override for the `Display` format
    fn display(
        &self,
        error: &(dyn StdError + 'static),
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        write!(f, "{}", error)?;

        if f.alternate() {
            for cause in crate::chain::Chain::new(error).skip(1) {
                write!(f, ": {}", cause)?;
            }
        }

        Ok(())
    }
}

/// The default provided error report handler for `eyre::Report`.
///
/// On nightly this supports conditionally capturing a `std::backtrace::Backtrace` if the source
/// error did not already capture one.
#[allow(dead_code)]
pub struct DefaultHandler {
    backtrace: Option<Backtrace>,
}

impl DefaultHandler {
    #[allow(unused_variables)]
    fn default_with(error: &(dyn StdError + 'static)) -> Box<dyn EyreHandler> {
        let backtrace = backtrace_if_absent!(error);

        Box::new(Self { backtrace })
    }
}

impl core::fmt::Debug for DefaultHandler {
    fn fmt(&self, f: &mut core::fmt::Formatter<'_>) -> core::fmt::Result {
        f.debug_struct("DefaultHandler")
            .field(
                "backtrace",
                match &self.backtrace {
                    Some(_) => &"Some(Backtrace { ... })",
                    None => &"None",
                },
            )
            .finish()
    }
}

impl EyreHandler for DefaultHandler {
    fn debug(
        &self,
        error: &(dyn StdError + 'static),
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        use core::fmt::Write as _;

        if f.alternate() {
            return core::fmt::Debug::fmt(error, f);
        }

        write!(f, "{}", error)?;

        if let Some(cause) = error.source() {
            write!(f, "\n\nCaused by:")?;
            let multiple = cause.source().is_some();
            for (n, error) in crate::chain::Chain::new(cause).enumerate() {
                writeln!(f)?;
                if multiple {
                    write!(indenter::indented(f).ind(n), "{}", error)?;
                } else {
                    write!(indenter::indented(f), "{}", error)?;
                }
            }
        }

        #[cfg(backtrace)]
        {
            use std::backtrace::BacktraceStatus;

            let backtrace = self
                .backtrace
                .as_ref()
                .or_else(|| error.backtrace())
                .expect("backtrace capture failed");
            if let BacktraceStatus::Captured = backtrace.status() {
                write!(f, "\n\nStack backtrace:\n{}", backtrace)?;
            }
        }

        Ok(())
    }
}

/// Iterator of a chain of source errors.
///
/// This type is the iterator returned by [`Report::chain`].
///
/// # Example
///
/// ```
/// use eyre::Report;
/// use std::io;
///
/// pub fn underlying_io_error_kind(error: &Report) -> Option<io::ErrorKind> {
///     for cause in error.chain() {
///         if let Some(io_error) = cause.downcast_ref::<io::Error>() {
///             return Some(io_error.kind());
///         }
///     }
///     None
/// }
/// ```
#[derive(Clone)]
#[allow(missing_debug_implementations)]
pub struct Chain<'a> {
    state: crate::chain::ChainState<'a>,
}

/// `Result<T, Error>`
///
/// This is a reasonable return type to use throughout your application but also for `fn main`; if
/// you do, failures will be printed along with a backtrace if one was captured.
///
/// `eyre::Result` may be used with one *or* two type parameters.
///
/// ```rust
/// use eyre::Result;
///
/// # const IGNORE: &str = stringify! {
/// fn demo1() -> Result<T> {...}
///            // ^ equivalent to std::result::Result<T, eyre::Error>
///
/// fn demo2() -> Result<T, OtherError> {...}
///            // ^ equivalent to std::result::Result<T, OtherError>
/// # };
/// ```
///
/// # Example
///
/// ```
/// # pub trait Deserialize {}
/// #
/// # mod serde_json {
/// #     use super::Deserialize;
/// #     use std::io;
/// #
/// #     pub fn from_str<T: Deserialize>(json: &str) -> io::Result<T> {
/// #         unimplemented!()
/// #     }
/// # }
/// #
/// # #[derive(Debug)]
/// # struct ClusterMap;
/// #
/// # impl Deserialize for ClusterMap {}
/// #
/// use eyre::Result;
///
/// fn main() -> Result<()> {
///     # return Ok(());
///     let config = std::fs::read_to_string("cluster.json")?;
///     let map: ClusterMap = serde_json::from_str(&config)?;
///     println!("cluster info: {:#?}", map);
///     Ok(())
/// }
/// ```
pub type Result<T, E = Report> = core::result::Result<T, E>;

/// Provides the `wrap_err` method for `Result`.
///
/// This trait is sealed and cannot be implemented for types outside of
/// `eyre`.
///
/// # Example
///
/// ```
/// use eyre::{WrapErr, Result};
/// use std::fs;
/// use std::path::PathBuf;
///
/// pub struct ImportantThing {
///     path: PathBuf,
/// }
///
/// impl ImportantThing {
///     # const IGNORE: &'static str = stringify! {
///     pub fn detach(&mut self) -> Result<()> {...}
///     # };
///     # fn detach(&mut self) -> Result<()> {
///     #     unimplemented!()
///     # }
/// }
///
/// pub fn do_it(mut it: ImportantThing) -> Result<Vec<u8>> {
///     it.detach().wrap_err("Failed to detach the important thing")?;
///
///     let path = &it.path;
///     let content = fs::read(path)
///         .wrap_err_with(|| format!("Failed to read instrs from {}", path.display()))?;
///
///     Ok(content)
/// }
/// ```
///
/// When printed, the outermost error would be printed first and the lower
/// level underlying causes would be enumerated below.
///
/// ```console
/// Error: Failed to read instrs from ./path/to/instrs.json
///
/// Caused by:
///     No such file or directory (os error 2)
/// ```
///
/// # Wrapping Types That Don't impl `Error` (e.g. `&str` and `Box<dyn Error>`)
///
/// Due to restrictions for coherence `Report` cannot impl `From` for types that don't impl
/// `Error`. Attempts to do so will give "this type might implement Error in the future" as an
/// error. As such, `wrap_err`, which uses `From` under the hood, cannot be used to wrap these
/// types. Instead we encourage you to use the combinators provided for `Result` in `std`/`core`.
///
/// For example, instead of this:
///
/// ```rust,compile_fail
/// use std::error::Error;
/// use eyre::{WrapErr, Report};
///
/// fn wrap_example(err: Result<(), Box<dyn Error + Send + Sync + 'static>>) -> Result<(), Report> {
///     err.wrap_err("saw a downstream error")
/// }
/// ```
///
/// We encourage you to write this:
///
/// ```rust
/// use std::error::Error;
/// use eyre::{WrapErr, Report, eyre};
///
/// fn wrap_example(err: Result<(), Box<dyn Error + Send + Sync + 'static>>) -> Result<(), Report> {
///     err.map_err(|e| eyre!(e)).wrap_err("saw a downstream error")
/// }
/// ```
///
/// # Effect on downcasting
///
/// After attaching a message of type `D` onto an error of type `E`, the resulting
/// `eyre::Error` may be downcast to `D` **or** to `E`.
///
/// That is, in codebases that rely on downcasting, Eyre's wrap_err supports
/// both of the following use cases:
///
///   - **Attaching messages whose type is insignificant onto errors whose type
///     is used in downcasts.**
///
///     In other error libraries whose wrap_err is not designed this way, it can
///     be risky to introduce messages to existing code because new message might
///     break existing working downcasts. In Eyre, any downcast that worked
///     before adding the message will continue to work after you add a message, so
///     you should freely wrap errors wherever it would be helpful.
///
///     ```
///     # use eyre::bail;
///     # use thiserror::Error;
///     #
///     # #[derive(Error, Debug)]
///     # #[error("???")]
///     # struct SuspiciousError;
///     #
///     # fn helper() -> Result<()> {
///     #     bail!(SuspiciousError);
///     # }
///     #
///     use eyre::{WrapErr, Result};
///
///     fn do_it() -> Result<()> {
///         helper().wrap_err("Failed to complete the work")?;
///         # const IGNORE: &str = stringify! {
///         ...
///         # };
///         # unreachable!()
///     }
///
///     fn main() {
///         let err = do_it().unwrap_err();
///         if let Some(e) = err.downcast_ref::<SuspiciousError>() {
///             // If helper() returned SuspiciousError, this downcast will
///             // correctly succeed even with the message in between.
///             # return;
///         }
///         # panic!("expected downcast to succeed");
///     }
///     ```
///
///   - **Attaching message whose type is used in downcasts onto errors whose
///     type is insignificant.**
///
///     Some codebases prefer to use machine-readable messages to categorize
///     lower level errors in a way that will be actionable to higher levels of
///     the application.
///
///     ```
///     # use eyre::bail;
///     # use thiserror::Error;
///     #
///     # #[derive(Error, Debug)]
///     # #[error("???")]
///     # struct HelperFailed;
///     #
///     # fn helper() -> Result<()> {
///     #     bail!("no such file or directory");
///     # }
///     #
///     use eyre::{WrapErr, Result};
///
///     fn do_it() -> Result<()> {
///         helper().wrap_err(HelperFailed)?;
///         # const IGNORE: &str = stringify! {
///         ...
///         # };
///         # unreachable!()
///     }
///
///     fn main() {
///         let err = do_it().unwrap_err();
///         if let Some(e) = err.downcast_ref::<HelperFailed>() {
///             // If helper failed, this downcast will succeed because
///             // HelperFailed is the message that has been attached to
///             // that error.
///             # return;
///         }
///         # panic!("expected downcast to succeed");
///     }
///     ```
pub trait WrapErr<T, E>: context::private::Sealed {
    /// Wrap the error value with a new adhoc error
    fn wrap_err<D>(self, msg: D) -> Result<T, Report>
    where
        D: Display + Send + Sync + 'static;

    /// Wrap the error value with a new adhoc error that is evaluated lazily
    /// only once an error does occur.
    fn wrap_err_with<D, F>(self, f: F) -> Result<T, Report>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D;

    /// Compatibility re-export of wrap_err for interopt with `anyhow`
    fn context<D>(self, msg: D) -> Result<T, Report>
    where
        D: Display + Send + Sync + 'static;

    /// Compatibility re-export of wrap_err_with for interopt with `anyhow`
    fn with_context<D, F>(self, f: F) -> Result<T, Report>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D;
}

/// Provides the `context` method for `Option` when porting from `anyhow`
///
/// This trait is sealed and cannot be implemented for types outside of
/// `eyre`.
///
/// ## Why Doesn't `Eyre` impl `WrapErr` for `Option`?
///
/// `eyre` doesn't impl `WrapErr` for `Option` because `wrap_err` implies that you're creating a
/// new error that saves the previous error as its `source`. Calling `wrap_err` on an `Option` is
/// meaningless because there is no source error. `anyhow` avoids this issue by using a different
/// mental model where you're adding "context" to an error, though this not a mental model for
/// error handling that `eyre` agrees with.
///
/// Instead, `eyre` encourages users to think of each error as distinct, where the previous error
/// is the context being saved by the new error, which is backwards compared to anyhow's model. In
/// this model you're encouraged to use combinators provided by `std` for `Option` to convert an
/// option to a `Result`
///
/// # Example
///
/// Instead of:
///
/// ```rust
/// use eyre::ContextCompat;
///
/// fn get_thing(mut things: impl Iterator<Item = u32>) -> eyre::Result<u32> {
///     things
///         .find(|&thing| thing == 42)
///         .context("the thing wasnt in the list")
/// }
/// ```
///
/// We encourage you to use this:
///
/// ```rust
/// use eyre::eyre;
///
/// fn get_thing(mut things: impl Iterator<Item = u32>) -> eyre::Result<u32> {
///     things
///         .find(|&thing| thing == 42)
///         .ok_or_else(|| eyre!("the thing wasnt in the list"))
/// }
/// ```
pub trait ContextCompat<T>: context::private::Sealed {
    /// Compatibility version of `wrap_err` for creating new errors with new source on `Option`
    /// when porting from `anyhow`
    fn context<D>(self, msg: D) -> Result<T, Report>
    where
        D: Display + Send + Sync + 'static;

    /// Compatibility version of `wrap_err_with` for creating new errors with new source on `Option`
    /// when porting from `anyhow`
    fn with_context<D, F>(self, f: F) -> Result<T, Report>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D;

    /// Compatibility re-export of `context` for porting from `anyhow` to `eyre`
    fn wrap_err<D>(self, msg: D) -> Result<T, Report>
    where
        D: Display + Send + Sync + 'static;

    /// Compatibility re-export of `with_context` for porting from `anyhow` to `eyre`
    fn wrap_err_with<D, F>(self, f: F) -> Result<T, Report>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D;
}

// Not public API. Referenced by macro-generated code.
#[doc(hidden)]
pub mod private {
    use crate::Report;
    use core::fmt::{Debug, Display};

    pub use core::result::Result::Err;

    #[doc(hidden)]
    pub mod kind {
        pub use crate::kind::{AdhocKind, TraitKind};

        pub use crate::kind::BoxedKind;
    }

    pub fn new_adhoc<M>(message: M) -> Report
    where
        M: Display + Debug + Send + Sync + 'static,
    {
        Report::from_adhoc(message)
    }
}
