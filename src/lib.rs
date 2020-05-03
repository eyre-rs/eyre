//! This library provides [`eyre::Report`][Report], a trait object based error
//! type for easy idiomatic error handling in Rust applications.
//!
//! This crate is a fork of [`anyhow`] by @dtolnay with a support for customized `Reports`. For
//! more details on customization checkout the docs on [`eyre::EyreContext`]. For an example on how
//! to implement a custom context check out [`stable-eyre`] which implements a minimal custom
//! context for capturing backtraces on stable.
//!
//! ```toml
//! [dependencies]
//! eyre = "0.3"
//! ```
//! # Details
//!
//! - Use `Result<T, eyre::Report>`, or equivalently `eyre::Result<T>`, as
//!   the return type of any fallible function.
//!
//!   Within the function, use `?` to easily propagate any error that implements
//!   the `std::error::Report` trait.
//!
//!   ```
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
//! - Create new errors from messages to help the person troubleshooting the error understand where
//! things went wrong. A low-level error like "No such file or directory" can be annoying to
//! directly and often benefit from being wrapped with higher level error messages.
//!
//!   ```
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
//!   ```
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
//!   error if the underlying error type does not already provide its own. In
//!   order to see backtraces, they must be enabled through the environment
//!   variables described in [`std::backtrace`]:
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
//!   including ones defined in your crate. We do not bundle a `derive(Error)`
//!   macro but you can write the impls yourself or use a standalone macro like
//!   [thiserror].
//!
//!   [thiserror]: https://github.com/dtolnay/thiserror
//!
//!   ```
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
//!   ```
//!   # use eyre::{eyre, Result};
//!   #
//!   # fn demo() -> Result<()> {
//!   #     let missing = "...";
//!   return Err(eyre!("Missing attribute: {}", missing));
//!   #     Ok(())
//!   # }
//!   ```
//!
//! # No-std support
//!
//! In no_std mode, the same API is almost all available and works the same way.
//! To depend on Eyre in no_std mode, disable our default enabled "std"
//! feature in Cargo.toml. A global allocator is required.
//!
//! ```toml
//! [dependencies]
//! eyre = { version = "0.3", default-features = false }
//! ```
//!
//! Since the `?`-based error conversions would normally rely on the
//! `std::error::Report` trait which is only available through std, no_std mode
//! will require an explicit `.map_err(Report::msg)` when working with a
//! non-Eyre error type inside a function that returns Eyre's error type.
//!
//! # Compatibility with `anyhow`
//!
//! This crate does its best to be usable as a drop in replacement of `anyhow` and
//! vice-versa by `re-exporting` all of the renamed APIs with the names used in
//! `anyhow`.
//!
//! It is not 100% compatible because there are some cases where `eyre` encounters
//! type inference errors but it should mostly work as a drop in replacement.
//! Specifically, the following works in anyhow:
//!
//! ```rust,compile_fail
//! // Works
//! let val = get_optional_val.ok_or_else(|| anyhow!("failed to get value")).unwrap();
//! ```
//!
//! Where as with `eyre!` this will fail due to being unable to infer the type for
//! the Context parameter. The solution to this problem, should you encounter it,
//! is to give the compiler a hint for what type it should be resolving to, either
//! via your return type or a type annotation.
//!
//! ```rust,compile_fail
//! // Broken
//! let val = get_optional_val.ok_or_else(|| eyre!("failed to get value")).unwrap();
//!
//! // Works
//! let val: Report = get_optional_val.ok_or_else(|| eyre!("failed to get value")).unwrap();
//! ```
//! [Report]: https://docs.rs/eyre/*/eyre/struct.Report.html
//! [`eyre::EyreContext`]: https://docs.rs/eyre/*/eyre/trait.EyreContext.html
//! [`eyre::WrapErr`]: https://docs.rs/eyre/*/eyre/trait.WrapErr.html
//! [`anyhow::Context`]: https://docs.rs/anyhow/*/anyhow/trait.Context.html
//! [`anyhow`]: https://github.com/dtolnay/anyhow
//! [`tracing_error::SpanTrace`]: https://docs.rs/tracing-error/*/tracing_error/struct.SpanTrace.html
//! [`stable_eyre`]: https://docs.rs/stable-eyre
#![doc(html_root_url = "https://docs.rs/eyre/0.3.10")]
#![cfg_attr(backtrace, feature(backtrace))]
#![cfg_attr(doc_cfg, feature(doc_cfg))]
#![cfg_attr(not(feature = "std"), no_std)]
#![allow(
    clippy::needless_doctest_main,
    clippy::new_ret_no_self,
    clippy::wrong_self_convention
)]

mod alloc {
    #[cfg(not(feature = "std"))]
    extern crate alloc;

    #[cfg(not(feature = "std"))]
    pub use alloc::boxed::Box;

    #[cfg(feature = "std")]
    pub use std::boxed::Box;

    #[cfg(not(feature = "std"))]
    pub use alloc::string::String;

    #[cfg(feature = "std")]
    pub use std::string::String;
}

#[macro_use]
mod backtrace;
mod chain;
mod context;
mod error;
mod fmt;
mod kind;
mod macros;
mod wrapper;

use crate::alloc::Box;
use crate::backtrace::Backtrace;
use crate::error::ErrorImpl;
use core::any::{Any, TypeId};
use core::fmt::Display;
use core::mem::ManuallyDrop;

#[cfg(not(feature = "std"))]
use core::fmt::Debug;

#[cfg(feature = "std")]
use std::error::Error as StdError;

#[cfg(not(feature = "std"))]
pub trait StdError: Debug + Display {
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        None
    }
}

pub use eyre as format_err;
/// Compatibility re-export of `eyre` for interopt with `anyhow`
pub use eyre as anyhow;
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
///    2: eyre::context::<impl eyre::WrapErr<T,E,C> for core::result::Result<T,E>>::wrap_err_with
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
pub struct Report<C = DefaultContext>
where
    C: EyreContext,
{
    inner: ManuallyDrop<Box<ErrorImpl<(), C>>>,
}

/// Context trait for customizing `eyre::Report`
///
/// ## Customization
///
/// In order to insert your own custom context type you must first implement the
/// `eyre::EyreContext` trait.
///
/// Once you've defined a custom Context type you can use it throughout your
/// application by defining a type alias.
///
/// ```rust
/// use backtrace::Backtrace;
/// use eyre::EyreContext;
/// # use eyre::Chain;
/// # use std::error::Error;
///
/// pub struct Context {
///     backtrace: Backtrace,
/// }
///
/// impl EyreContext for Context {
///     // ...
/// #     #[allow(unused_variables)]
/// #     fn default(error: &(dyn Error + 'static)) -> Self {
/// #         let backtrace = Backtrace::new();
/// #         Self { backtrace }
/// #     }
/// #     fn debug(
/// #         &self,
/// #         error: &(dyn Error + 'static),
/// #         f: &mut core::fmt::Formatter<'_>,
/// #     ) -> core::fmt::Result {
/// #         use core::fmt::Write as _;
/// #         if f.alternate() {
/// #             return core::fmt::Debug::fmt(error, f);
/// #         }
/// #         write!(f, "{}", error)?;
/// #         if let Some(cause) = error.source() {
/// #             write!(f, "\n\nCaused by:")?;
/// #             let multiple = cause.source().is_some();
/// #             for (n, error) in Chain::new(cause).enumerate() {
/// #                 writeln!(f)?;
/// #                 if multiple {
/// #                     write!(indenter::Indented::numbered(f, n), "{}", error)?;
/// #                 } else {
/// #                     write!(indenter::Indented::new(f), "{}", error)?;
/// #                 }
/// #             }
/// #         }
/// #         let backtrace = &self.backtrace;
/// #         write!(f, "\n\nStack backtrace:\n{:?}", backtrace)?;
/// #         Ok(())
/// #     }
/// }
///
/// type Report = eyre::Report<Context>;
/// type Result<T, E = eyre::Report<Context>> = core::result::Result<T, E>;
/// ```
pub trait EyreContext: Sized + Send + Sync + 'static {
    /// Default construct a `Context` when constructing a `Report`.
    ///
    /// This method provides a reference to the error being wrapped to support conditional
    /// capturing of context like `backtrace` depending on whether the source error already
    /// captured one.
    ///
    /// # Example
    ///
    /// ```rust
    /// use backtrace::Backtrace;
    /// use eyre::EyreContext;
    /// # use eyre::Chain;
    /// use std::error::Error;
    ///
    /// pub struct Context {
    ///     backtrace: Backtrace,
    /// }
    ///
    /// impl EyreContext for Context {
    /// #     #[allow(unused_variables)]
    ///     fn default(error: &(dyn Error + 'static)) -> Self {
    ///         let backtrace = Backtrace::new();
    ///
    ///         Self { backtrace }
    ///     }
    ///
    ///     // ...
    /// #     fn debug(
    /// #         &self,
    /// #         error: &(dyn Error + 'static),
    /// #         f: &mut core::fmt::Formatter<'_>,
    /// #     ) -> core::fmt::Result {
    /// #         use core::fmt::Write as _;
    /// #         if f.alternate() {
    /// #             return core::fmt::Debug::fmt(error, f);
    /// #         }
    /// #         write!(f, "{}", error)?;
    /// #         if let Some(cause) = error.source() {
    /// #             write!(f, "\n\nCaused by:")?;
    /// #             let multiple = cause.source().is_some();
    /// #             for (n, error) in Chain::new(cause).enumerate() {
    /// #                 writeln!(f)?;
    /// #                 if multiple {
    /// #                     write!(indenter::Indented::numbered(f, n), "{}", error)?;
    /// #                 } else {
    /// #                     write!(indenter::Indented::new(f), "{}", error)?;
    /// #                 }
    /// #             }
    /// #         }
    /// #         let backtrace = &self.backtrace;
    /// #         write!(f, "\n\nStack backtrace:\n{:?}", backtrace)?;
    /// #         Ok(())
    /// #     }
    /// }
    /// ```
    fn default(err: &(dyn StdError + 'static)) -> Self;

    /// Define the report format
    ///
    /// Used to override the report format of `eyre::Report`
    ///
    /// # Example
    ///
    /// ```rust
    /// use backtrace::Backtrace;
    /// use eyre::EyreContext;
    /// use eyre::Chain;
    /// use std::error::Error;
    ///
    /// pub struct Context {
    ///     backtrace: Backtrace,
    /// }
    ///
    /// impl EyreContext for Context {
    /// #     #[allow(unused_variables)]
    /// #     fn default(error: &(dyn Error + 'static)) -> Self {
    /// #         let backtrace = Backtrace::new();
    /// #         Self { backtrace }
    /// #     }
    ///     // ...
    ///
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
    ///                     write!(indenter::Indented::numbered(f, n), "{}", error)?;
    ///                 } else {
    ///                     write!(indenter::Indented::new(f), "{}", error)?;
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

    /// Member access function
    ///
    /// The main reason to implement this fn is to provide support for `eyre::Report::backtrace`
    /// which will call this fn on `nightly` when attempting access the captured
    /// `std::backtrace::Backtrace`
    ///
    /// # Example
    fn member_ref(&self, _typeid: TypeId) -> Option<&dyn Any> {
        None
    }

    #[doc(hidden)]
    fn member_mut(&mut self, _typeid: TypeId) -> Option<&mut dyn Any> {
        None
    }

    #[doc(hidden)]
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

/// The default provided context for `eyre::Report`.
///
/// On nightly this supports conditionally capturing a `std::backtrace::Backtrace` if the source
/// error did not already capture one.
pub struct DefaultContext {
    backtrace: Option<Backtrace>,
}

impl EyreContext for DefaultContext {
    #[allow(unused_variables)]
    fn default(error: &(dyn StdError + 'static)) -> Self {
        let backtrace = backtrace_if_absent!(error);

        Self { backtrace }
    }

    fn member_ref(&self, typeid: TypeId) -> Option<&dyn Any> {
        if typeid == TypeId::of::<Backtrace>() {
            self.backtrace.as_ref().map(|b| b as &dyn Any)
        } else {
            None
        }
    }

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
                    write!(indenter::Indented::numbered(f, n), "{}", error)?;
                } else {
                    write!(indenter::Indented::new(f), "{}", error)?;
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
#[cfg(feature = "std")]
#[derive(Clone)]
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
pub type Result<T, E = Report<DefaultContext>> = core::result::Result<T, E>;

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
pub trait WrapErr<T, E, C>: context::private::Sealed<C>
where
    C: EyreContext,
{
    /// Wrap the error value with a new adhoc error
    fn wrap_err<D>(self, msg: D) -> Result<T, Report<C>>
    where
        D: Display + Send + Sync + 'static;

    /// Wrap the error value with a new adhoc error that is evaluated lazily
    /// only once an error does occur.
    fn wrap_err_with<D, F>(self, f: F) -> Result<T, Report<C>>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D;

    /// Compatibility re-export of wrap_err for interopt with `anyhow`
    fn context<D>(self, msg: D) -> Result<T, Report<C>>
    where
        D: Display + Send + Sync + 'static;

    /// Compatibility re-export of wrap_err_with for interopt with `anyhow`
    fn with_context<D, F>(self, f: F) -> Result<T, Report<C>>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D;
}

// Not public API. Referenced by macro-generated code.
#[doc(hidden)]
pub mod private {
    use crate::{EyreContext, Report};
    use core::fmt::{Debug, Display};

    //     #[cfg(backtrace)]
    //     use std::backtrace::Backtrace;

    pub use core::result::Result::Err;

    #[doc(hidden)]
    pub mod kind {
        pub use crate::kind::{AdhocKind, TraitKind};

        #[cfg(feature = "std")]
        pub use crate::kind::BoxedKind;
    }

    pub fn new_adhoc<M, C>(message: M) -> Report<C>
    where
        C: EyreContext,
        M: Display + Debug + Send + Sync + 'static,
    {
        Report::from_adhoc(message)
    }
}
