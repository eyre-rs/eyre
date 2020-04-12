//! A tutorial on how to customize eyre by overriding the Context parameter
//!
//! # Introduction
//!
//! In this tutorial we will go through the process of implementing a custom context step by step:
//!
//! * Start with defining an empty context that captures nothing
//! * Add support for capturing backtraces
//!     * skip backtrace capture if our source captured one
//! * Add support for a http status codes
//!     * Implement a setter for Result<T, Report> with a new Trait
//!
//! # A Basic Context
//!
//! The simplest context is an empty context, so lets start there, here's what our context looks
//! like:
//!
//! ```rust
//! struct CustomContext;
//! ```
//!
//! Now this context doesn't add much context, but it does have the advantage of removing context,
//! aka the backtrace, that may be unnecessary. To use this backtrace we need to implement
//! `EyreContext` which provides the necessary interface for eyre construct and use your context.
//!
//! First, we have to impl `default` so eyre knows how to construct your context when it creates
//! your `Report from a `std::error::Error`:
//!
//!
//! ```rust
//! fn default(_: &(dyn std::error::Error + 'static)) -> Self {
//!     Self
//! }
//! ```
//!
//! And next we need to implement `debug` to tell eyre how to format the final report including
//! your custom context:
//!
//! ```rust
//! use eyre::{Chain, Indenter};
//!
//! fn debug(
//!     &self,
//!     error: &(dyn std::error::Error + 'static),
//!     f: &mut core::fmt::Formatter<'_>,
//! ) -> core::fmt::Result {
//!     if f.alternate() {
//!         return core::fmt::Debug::fmt(error, f);
//!     }
//!
//!     let errors = Chain::new(error).enumerate();
//!
//!     for (n, error) in errors {
//!         writeln!(f)?;
//!         write!(Indented::numbered(f, n), "{}", error)?;
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! Things to note, we're using some utilities provided by `eyre` for writing error reports,
//!
//! - `Chain`: iterator for errors while the `chain` fn on the error trait is still unstable.
//! - `Indenter`: a wrapper for formatters that inserts indentation after newlines for handling
//! multi line error messages nicely
//!
//! Only one step remains, we need to override `eyre::Report`'s context parameter, which we will do
//! with a type alias.
//!
//! ```rust
//! pub(crate) type Report = eyre::Report<CustomContext>;
//! ```
//!
//! And we're done! Here's what our final do nothing implementation of a custom context looks like:
//!
//! ```rust
//! use eyre::Chain;
//! use indenter::Indented;
//!
//! pub(crate) type Report = eyre::Report<CustomContext>;
//!
//! struct CustomContext;
//!
//! impl eyre::EyreContext for CustomContext {
//!     fn default(_: &(dyn std::error::Error + 'static)) -> Self {
//!         Self
//!     }
//!
//!     fn debug(
//!         &self,
//!         error: &(dyn std::error::Error + 'static),
//!         f: &mut core::fmt::Formatter<'_>,
//!     ) -> core::fmt::Result {
//!         if f.alternate() {
//!             return core::fmt::Debug::fmt(error, f);
//!         }
//!
//!         let errors = Chain::new(error).enumerate();
//!
//!         for (n, error) in errors {
//!             writeln!(f)?;
//!             write!(Indented::numbered(f, n), "{}", error)?;
//!         }
//!
//!         Ok(())
//!     }
//! }
//! ```
//!
//! # Lets Add Context to Our Context
//!
//! So now that we have the basic shape of a context and error report setup, let's go through the
//! steps of adding a backtrace, by the end of this section we will have a context that almost
//! identical to the `DefaultContext` provided by eyre and anyhow. The only difference between our
//! implementation and the provided ones is that we won't jump through the extra hoops to
//! conditionally disable backtraces when they're not available (e.g. we're on stable).
//!
//! To start we need to be able to store a backtrace in our context, lets modify our definition:
//!
//! ```rust
//! use std::backtrace::Backtrace;
//!
//! struct CustomContext {
//!     backtrace: Backtrace;
//! }
//! ```
//!
//! Next, lets modify the `default` fn to capture our backtrace when we first create our report
//! from a `std::error::Error`:
//!
//! ```rust
//! use std::backtrace::Backtrace;
//!
//! fn default(_: &(dyn std::error::Error + 'static)) -> Self {
//!     Self {
//!         backtrace: Backtrace::capture(),
//!     }
//! }
//! ```
//!
//! Now we have a backtrace, but our report still wont print it, lets edit `debug to print this
//! new member variable in our error report:
//!
//! ```rust
//! use eyre::{Chain, Indenter};
//! use std::backtrace::BacktraceStatus;
//!
//! fn debug(
//!     &self,
//!     error: &(dyn std::error::Error + 'static),
//!     f: &mut core::fmt::Formatter<'_>,
//! ) -> core::fmt::Result {
//!     if f.alternate() {
//!         return core::fmt::Debug::fmt(error, f);
//!     }
//!
//!     let errors = Chain::new(error).enumerate();
//!
//!     for (n, error) in errors {
//!         writeln!(f)?;
//!         write!(Indented::numbered(f, n), "{}", error)?;
//!     }
//!
//!     let backtrace = &self.backtrace;
//!
//!     if let BacktraceStatus::Captured = backtrace.status() {
//!         write!(f, "\n\nStack backtrace:\n{}", self.backtrace)?;
//!     }
//!
//!     Ok(())
//! }
//! ```
//!
//! And we're done! Well, not quite, we have a backtrace sure, but what if we want to capture a
//! backtrace before we convert our error into a `Report`? This will only ever print the backtrace
//! we captured for the report and ignore any backtraces captured by our sources.
//!
//! ## Changing Behavior Based on Our Source Error
//!
//! You may have noticed but there's an extra arg on `default` that we've been ignoring this whole
//! time, this arg is the key to doing things like conditionally capturing backtraces when we
//! construct our context, as it lets us inspect the shape of the inner error via the
//! `std::error::Error` interface.
//!
//! First, lets modify the struct so we can handle not capturing a `Backtrace`:
//!
//! ```rust
//! use std::backtrace::Backtrace;
//!
//! struct CustomContext {
//!     backtrace: Option<Backtrace>;
//! }
//! ```
//!
//! Next we update `default`:
//!
//! ```rust
//! use std::backtrace::Backtrace;
//!
//! fn default(inner: &(dyn std::error::Error + 'static)) -> Self {
//!     let backtrace = if Chain::new(source).all(|e| e.backtrace().is_none()) {
//!         Some(Backtrace::capture())
//!     } else {
//!         None
//!     };
//!
//!     Self { backtrace }
//! }
//! ```
//!
//! And finally we just have to update the part of `debug` that prints our backtrace:
//!
//! ```rust
//!  let backtrace = self
//!      .backtrace
//!      .as_ref()
//!      .or_else(|| Chain::new(error).rev().filter_map(|e| e.backtrace()).next())
//!      .expect("expected: backtrace is always captured");
//!
//! if let BacktraceStatus::Captured = backtrace.status() {
//!     write!(f, "\n\nStack backtrace:\n{}", self.backtrace)?;
//! }
//! ```
//!
//! And we're done! Here is what our final implementation looks like:
//!
//!
//! ```rust
//! use eyre::Chain;
//! use indenter::Indented;
//! use std::backtrace::{Backtrace, BacktraceStatus};
//!
//! pub(crate) type Report = eyre::Report<CustomContext>;
//!
//! struct CustomContext {
//!     backtrace: Option<Backtrace>;
//! }
//!
//! impl eyre::EyreContext for CustomContext {
//!     fn default(inner: &(dyn std::error::Error + 'static)) -> Self {
//!         let backtrace = if Chain::new(source).all(|e| e.backtrace().is_none()) {
//!             Some(Backtrace::capture())
//!         } else {
//!             None
//!         };
//!
//!         Self { backtrace }
//!     }
//!
//!     fn debug(
//!         &self,
//!         error: &(dyn std::error::Error + 'static),
//!         f: &mut core::fmt::Formatter<'_>,
//!     ) -> core::fmt::Result {
//!         if f.alternate() {
//!             return core::fmt::Debug::fmt(error, f);
//!         }
//!
//!         let errors = Chain::new(error).enumerate();
//!
//!         for (n, error) in errors {
//!             writeln!(f)?;
//!             write!(Indented::numbered(f, n), "{}", error)?;
//!         }
//!
//!         let backtrace = self
//!             .backtrace
//!             .as_ref()
//!             .or_else(|| Chain::new(error).rev().filter_map(|e| e.backtrace()).next())
//!             .expect("expected: backtrace is always captured");
//!
//!         if let BacktraceStatus::Captured = backtrace.status() {
//!             write!(f, "\n\nStack backtrace:\n{}", self.backtrace)?;
//!         }
//!
//!         Ok(())
//!     }
//! }
//! ```
//!
//! # An Actually Custom Context
//!
//! So now we've gone through two iterations of contexts that didn't really add much of anything to
//! the functionality of `Report`, let's try using this library for what its made for! Lets use our
//! `Report` for setting and reporting http status codes.
//!
//! First, lets do everything we've done in the previous steps, but all at once now that we've got
//! the hang of it:
//!
//! ```rust
//! use eyre::Chain;
//! use indenter::Indented;
//! use std::backtrace::{Backtrace, BacktraceStatus};
//! use your_favorite_http_framework::StatusCode;
//!
//! pub(crate) type Report = eyre::Report<CustomContext>;
//!
//! struct CustomContext {
//!     // store the status
//!     pub(crate) status: StatusCode,
//!     backtrace: Option<Backtrace>;
//! }
//!
//! impl eyre::EyreContext for CustomContext {
//!     fn default(inner: &(dyn std::error::Error + 'static)) -> Self {
//!         let backtrace = if Chain::new(source).all(|e| e.backtrace().is_none()) {
//!             Some(Backtrace::capture())
//!         } else {
//!             None
//!         };
//!
//!         Self {
//!             // set a default status
//!             status: StatusCode::INTERNAL_SERVER_ERROR,
//!             backtrace
//!         }
//!     }
//!
//!     fn debug(
//!         &self,
//!         error: &(dyn std::error::Error + 'static),
//!         f: &mut core::fmt::Formatter<'_>,
//!     ) -> core::fmt::Result {
//!         if f.alternate() {
//!             return core::fmt::Debug::fmt(error, f);
//!         }
//!
//!         // print our status on the first line
//!         write!(f, "{}", self.status)?;
//!
//!         let errors = Chain::new(error).enumerate();
//!
//!         for (n, error) in errors {
//!             writeln!(f)?;
//!             write!(Indented::numbered(f, n), "{}", error)?;
//!         }
//!
//!         let backtrace = self
//!             .backtrace
//!             .as_ref()
//!             .or_else(|| Chain::new(error).rev().filter_map(|e| e.backtrace()).next())
//!             .expect("expected: backtrace is always captured");
//!
//!         if let BacktraceStatus::Captured = backtrace.status() {
//!             write!(f, "\n\nStack backtrace:\n{}", self.backtrace)?;
//!         }
//!
//!         Ok(())
//!     }
//! }
//! ```
//!
//! This is great all, but now we just get INTERNAL_SERVER_ERROR every time, which isn't very
//! useful, lets add a helper trait of our own, similar to the provided `WrapErr` trait, for
//! setting the status.
//!
//! ```rust
//! pub(crate) trait Status {
//!     type Result;
//!     fn set_status(self, status: StatusCode) -> Self::Result;
//! }
//!
//! impl<T, E> Status for Result<T, E>
//! where
//!     E: Into<Report>,
//! {
//!     type Result = Result<T, Report>;
//!
//!     fn set_status(self, status: StatusCode) -> Self::Result {
//!         self.map_err(|e| {
//!             let mut reporter = e.into();
//!             reporter.context_mut().status = status;
//!             reporter
//!         })
//!     }
//! }
//! ```
//!
//! Now we can use our reporter to associate status codes with any error we throw with
//! INTERNAL_SERVER_ERROR as a hopefully reasonable default should we forget / not want to set the
//! status. Here's what our error handling api ends up looking like in practice:
//!
//! ```rust
//! pub(crate) async fn get(&self, timeframe: Timeframe) -> Result<Vec<u8>, Report> {
//!     let mut map = self.map.lock().unwrap();
//!     let entry = map
//!         .get_mut(&timeframe)
//!         .ok_or_else(|| eyre!("No entries for this timestamp"))
//!         .set_status(StatusCode::NOT_FOUND)?;
//!
//!     // ...
//! }
//! ```
//!
//! And we're done! Hopefully this gives you a solid understanding of how to work with `eyre`'s
//! custom context feature and how to use it effectively. If theres anything I missed please don't
//! hesitate to open an issue or reach out on twitter or discord.
