//! This library provides a custom [`eyre::EyreHandler`] type for usage with [`eyre`] that provides
//! a minimal error report with no additional context. Essentially the minimal implementation of an
//! error reporter.
//!
//! ## Setup
//!
//! Add the following to your toml file:
//!
//! ```toml
//! [dependencies]
//! simple-eyre = "0.3"
//! ```
//!
//! Then install the hook handler before constructing any `eyre::Report` types.
//!
//! # Example
//!
//! ```rust,should_panic
//! use simple_eyre::eyre::{eyre, WrapErr, Report};
//!
//! fn main() -> Result<(), Report> {
//!     simple_eyre::install()?;
//!
//!     let e: Report = eyre!("oh no this program is just bad!");
//!
//!     Err(e).wrap_err("usage example successfully experienced a failure")
//! }
//! ```
//!
//! [`eyre::EyreHandler`]: https://docs.rs/eyre/*/eyre/trait.EyreHandler.html
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
pub use eyre;
#[doc(hidden)]
pub use eyre::{Report, Result};

use eyre::EyreHandler;
use indenter::indented;
use std::error::Error;

/// A custom context type for minimal error reporting via `eyre`
#[derive(Debug)]
pub struct Handler;

impl EyreHandler for Handler {
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
            let errors = std::iter::successors(Some(cause), |e| (*e).source());

            for (n, error) in errors.enumerate() {
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

/// Install the `simple-eyre` hook as the global error report hook.
///
/// # Details
///
/// This function must be called to enable the customization of `eyre::Report`
/// provided by `simple-eyre`. This function should be called early, ideally
/// before any errors could be encountered.
///
/// Only the first install will succeed. Calling this function after another
/// report handler has been installed will cause an error. **Note**: This
/// function _must_ be called before any `eyre::Report`s are constructed to
/// prevent the default handler from being installed.
pub fn install() -> Result<()> {
    crate::eyre::set_hook(Box::new(move |_| Box::new(Handler)))?;

    Ok(())
}
