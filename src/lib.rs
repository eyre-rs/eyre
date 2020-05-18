//! This library provides a custom [`eyre::EyreContext`] type for usage with [`eyre`] that provides
//! a minimal error report with no additional context. Essentially the minimal implementation of an
//! error reporter.
//!
//! # Example
//!
//! ```rust,should_panic
//! use eyre::{eyre, WrapErr};
//! use simple_eyre::Report;
//!
//! fn main() -> Result<(), Report> {
//!     let e: Report = eyre!("oh no this program is just bad!");
//!
//!     Err(e).wrap_err("usage example successfully experienced a failure")
//! }
//! ```
//!
//! [`eyre::EyreContext`]: https://docs.rs/eyre/*/eyre/trait.EyreContext.html
//! [`eyre`]: https://docs.rs/eyre
#![doc(html_root_url = "https://docs.rs/simple-eyre/0.2.0")]
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
use eyre::Chain;
use eyre::EyreContext;
use indenter::indented;
use std::error::Error;

/// A custom context type for minimal error reporting via `eyre`
#[derive(Debug)]
pub struct Context;

impl EyreContext for Context {
    #[allow(unused_variables)]
    fn default(error: &(dyn Error + 'static)) -> Self {
        Self
    }

    fn debug(
        &self,
        error: &(dyn Error + 'static),
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
            for (n, error) in Chain::new(cause).enumerate() {
                writeln!(f)?;
                if multiple {
                    write!(indented(f).ind(n), "{}", error)?;
                } else {
                    write!(indented(f), "{}", error)?;
                }
            }
        }

        Ok(())
    }
}

/// A type alias for `eyre::Report<simple_eyre::Context>`
///
/// # Example
///
/// ```rust
/// use simple_eyre::Report;
///
/// # struct Config;
/// fn try_thing(path: &str) -> Result<Config, Report> {
///     // ...
/// # Ok(Config)
/// }
/// ```
pub type Report = eyre::Report<Context>;

/// A type alias for `Result<T, simple_eyre::Report>`
///
/// # Example
///
///```
/// fn main() -> simple_eyre::Result<()> {
///
///     // ...
///
///     Ok(())
/// }
/// ```
pub type Result<T, E = Report> = core::result::Result<T, E>;
