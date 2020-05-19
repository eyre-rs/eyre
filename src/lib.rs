//! A custom context for the [`eyre`] crate for colorful error reports, suggestions,
//! and [`tracing-error`] support.
//!
//! ## Setup
//!
//! Add the following to your toml file:
//!
//! ```toml
//! [dependencies]
//! eyre = "0.4"
//! color-eyre = "0.3"
//! ```
//!
//! And then import the type alias from color-eyre for [`eyre::Report`] or [`eyre::Result`].
//!
//! ```rust
//! use color_eyre::Report;
//!
//! // or
//!
//! fn example() -> color_eyre::Result<()> {
//!     # Ok(())
//!     // ...
//! }
//! ```
//!
//! ### Disabling tracing support
//!
//! If you don't plan on using `tracing_error` and `SpanTrace` you can disable the
//! tracing integration to cut down on unused dependencies:
//!
//! ```toml
//! [dependencies]
//! eyre = "0.4"
//! color-eyre = { version = "0.3", default-features = false }
//! ```
//!
//! ## Example
//!
//! ```rust,should_panic
//! use color_eyre::{Help, Report};
//! use eyre::WrapErr;
//! use tracing::{info, instrument};
//!
//! #[instrument]
//! fn main() -> Result<(), Report> {
//!     #[cfg(feature = "capture-spantrace")]
//!     install_tracing();
//!
//!     Ok(read_config()?)
//! }
//!
//! #[cfg(feature = "capture-spantrace")]
//! fn install_tracing() {
//!     use tracing_error::ErrorLayer;
//!     use tracing_subscriber::prelude::*;
//!     use tracing_subscriber::{fmt, EnvFilter};
//!
//!     let fmt_layer = fmt::layer().with_target(false);
//!     let filter_layer = EnvFilter::try_from_default_env()
//!         .or_else(|_| EnvFilter::try_new("info"))
//!         .unwrap();
//!
//!     tracing_subscriber::registry()
//!         .with(filter_layer)
//!         .with(fmt_layer)
//!         .with(ErrorLayer::default())
//!         .init();
//! }
//!
//! #[instrument]
//! fn read_file(path: &str) -> Result<(), Report> {
//!     info!("Reading file");
//!     Ok(std::fs::read_to_string(path).map(drop)?)
//! }
//!
//! #[instrument]
//! fn read_config() -> Result<(), Report> {
//!     read_file("fake_file")
//!         .wrap_err("Unable to read config")
//!         .suggestion("try using a file that exists next time")
//! }
//! ```
//!
//! ## Minimal Report Format
//!
//! ![minimal report format](https://raw.githubusercontent.com/yaahc/color-eyre/master/pictures/minimal.png)
//!
//! ## Short Report Format (with `RUST_LIB_BACKTRACE=1`)
//!
//! ![short report format](https://raw.githubusercontent.com/yaahc/color-eyre/master/pictures/short.png)
//!
//! ## Full Report Format (with `RUST_LIB_BACKTRACE=full`)
//!
//! ![full report format](https://raw.githubusercontent.com/yaahc/color-eyre/master/pictures/full.png)
//!
//! ## Explanation
//!
//! This crate works by defining a `Context` type which implements [`eyre::EyreContext`]
//! and a pair of type aliases for setting this context type as the parameter of
//! [`eyre::Report`].
//!
//! ```rust
//! use color_eyre::Context;
//!
//! pub type Report = eyre::Report<Context>;
//! pub type Result<T, E = Report> = core::result::Result<T, E>;
//! ```
//!
//! Please refer to the [`Context`] type's docs for more details about its feature set.
//!
//! ## Features
//!
//! - captures a [`backtrace::Backtrace`] and prints using [`color-backtrace`]
//! - captures a [`tracing_error::SpanTrace`] and prints using
//! [`color-spantrace`]
//! - Only capture SpanTrace by default for better performance.
//! - display source lines when `RUST_LIB_BACKTRACE=full` is set
//! - store help text via [`Help`] trait and display after final report
//! - custom `color-backtrace` configuration via `color_eyre::install`,
//!   such as adding custom filters
//!
//!
//! [`eyre`]: https://docs.rs/eyre
//! [`tracing-error`]: https://docs.rs/tracing-error
//! [`color-backtrace`]: https://docs.rs/color-backtrace
//! [`eyre::EyreContext`]: https://docs.rs/eyre/*/eyre/trait.EyreContext.html
//! [`backtrace::Backtrace`]: https://docs.rs/backtrace/*/backtrace/struct.Backtrace.html
//! [`tracing_error::SpanTrace`]: https://docs.rs/tracing-error/*/tracing_error/struct.SpanTrace.html
//! [`color-spantrace`]: https://github.com/yaahc/color-spantrace
//! [`Help`]: trait.Help.html
//! [`eyre::Report`]: https://docs.rs/eyre/*/eyre/struct.Report.html
//! [`eyre::Result`]: https://docs.rs/eyre/*/eyre/type.Result.html
//! [`Context`]: struct.Context.html
#![doc(html_root_url = "https://docs.rs/color-eyre/0.3.2")]
#![cfg_attr(docsrs, feature(doc_cfg))]
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
use ansi_term::Color::*;
use backtrace::Backtrace;
pub use color_backtrace::BacktracePrinter;
use eyre::*;
pub use help::Help;
use help::HelpInfo;
use indenter::{indented, Format};
use once_cell::sync::OnceCell;
#[cfg(feature = "capture-spantrace")]
use std::error::Error;
use std::{
    env,
    fmt::{self, Write as _},
    sync::atomic::{AtomicUsize, Ordering::SeqCst},
};
#[cfg(feature = "capture-spantrace")]
use tracing_error::{ExtractSpanTrace, SpanTrace, SpanTraceStatus};

mod help;

static CONFIG: OnceCell<BacktracePrinter> = OnceCell::new();

/// A custom context type for [`eyre::Report`] which provides colorful error
/// reports and [`tracing-error`] support.
///
/// This type is not intended to be used directly, prefer using it via the
/// [`color_eyre::Report`] and [`color_eyre::Result`] type aliases.
///
/// [`eyre::Report`]: https://docs.rs/eyre/0.3.8/eyre/struct.Report.html
/// [`tracing-error`]: https://docs.rs/tracing-error
/// [`color_eyre::Report`]: type.Report.html
/// [`color_eyre::Result`]: type.Result.html
#[derive(Debug)]
pub struct Context {
    backtrace: Option<Backtrace>,
    #[cfg(feature = "capture-spantrace")]
    span_trace: Option<SpanTrace>,
    help: Vec<HelpInfo>,
}

#[derive(Debug)]
struct InstallError;

impl Context {
    /// Return a reference to the captured `Backtrace` type
    ///
    /// # Examples
    ///
    /// Backtrace capture can be enabled with the `RUST_BACKTRACE` env variable:
    ///
    /// ```
    /// use color_eyre::Report;
    /// use eyre::eyre;
    ///
    /// std::env::set_var("RUST_BACKTRACE", "1");
    ///
    /// let report: Report = eyre!("an error occurred");
    /// assert!(report.context().backtrace().is_some());
    /// ```
    ///
    /// Alternatively, if you don't want backtraces to be printed on panic, you can use
    /// `RUST_LIB_BACKTRACE`:
    ///
    /// ```
    /// use color_eyre::Report;
    /// use eyre::eyre;
    ///
    /// std::env::set_var("RUST_LIB_BACKTRACE", "1");
    ///
    /// let report: Report = eyre!("an error occurred");
    /// assert!(report.context().backtrace().is_some());
    /// ```
    ///
    /// And if you don't want backtraces to be captured but you still want panics to print
    /// backtraces you can explicitly set `RUST_LIB_BACKTRACE` to 0:
    ///
    /// ```
    /// use color_eyre::Report;
    /// use eyre::eyre;
    ///
    /// std::env::set_var("RUST_BACKTRACE", "1");
    /// std::env::set_var("RUST_LIB_BACKTRACE", "0");
    ///
    /// let report: Report = eyre!("an error occurred");
    /// assert!(report.context().backtrace().is_none());
    /// ```
    ///
    pub fn backtrace(&self) -> Option<&Backtrace> {
        self.backtrace.as_ref()
    }

    /// Return a reference to the captured `SpanTrace` type
    ///
    /// # Examples
    ///
    /// SpanTraces are always captured by default:
    ///
    /// ```
    /// use color_eyre::Report;
    /// use eyre::eyre;
    ///
    /// let report: Report = eyre!("an error occurred");
    /// assert!(report.context().span_trace().is_some());
    /// ```
    ///
    /// However, `SpanTrace` is not captured if one of the source errors already captured a
    /// `SpanTrace` via [`tracing_error::TracedError`]:
    ///
    /// ```
    /// use color_eyre::Report;
    /// use eyre::eyre;
    /// use tracing_error::{TracedError, InstrumentError};
    ///
    /// #[derive(Debug)]
    /// struct SourceError;
    ///
    /// impl std::fmt::Display for SourceError {
    ///     fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
    ///         write!(f, "SourceError")
    ///     }
    /// }
    ///
    /// impl std::error::Error for SourceError {}
    ///
    /// let error = SourceError;
    ///
    /// // the type annotation here is unnecessary, I've only added it for demonstration purposes
    /// let error: TracedError<SourceError> = error.in_current_span();
    ///
    /// let report: Report = error.into();
    /// assert!(report.context().span_trace().is_none());
    /// ```
    ///
    /// [`tracing_error::TracedError`]: https://docs.rs/tracing-error/0.1.2/tracing_error/struct.TracedError.html
    #[cfg(feature = "capture-spantrace")]
    #[cfg_attr(docsrs, doc(cfg(feature = "capture-spantrace")))]
    pub fn span_trace(&self) -> Option<&SpanTrace> {
        self.span_trace.as_ref()
    }
}

impl EyreContext for Context {
    #[allow(unused_variables)]
    fn default(error: &(dyn std::error::Error + 'static)) -> Self {
        let backtrace = if backtrace_enabled() {
            Some(Backtrace::new())
        } else {
            None
        };

        #[cfg(feature = "capture-spantrace")]
        let span_trace = if get_deepest_spantrace(error).is_none() {
            Some(SpanTrace::capture())
        } else {
            None
        };

        Self {
            backtrace,
            #[cfg(feature = "capture-spantrace")]
            span_trace,
            help: Vec::new(),
        }
    }

    fn debug(
        &self,
        error: &(dyn std::error::Error + 'static),
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        if f.alternate() {
            return core::fmt::Debug::fmt(error, f);
        }

        #[cfg(feature = "capture-spantrace")]
        let errors = Chain::new(error)
            .filter(|e| e.span_trace().is_none())
            .enumerate();

        #[cfg(not(feature = "capture-spantrace"))]
        let errors = Chain::new(error).enumerate();

        let mut buf = String::new();
        for (n, error) in errors {
            writeln!(f)?;
            buf.clear();
            write!(&mut buf, "{}", error).unwrap();
            write!(indented(f).ind(n), "{}", Red.paint(&buf))?;
        }

        #[cfg(feature = "capture-spantrace")]
        {
            let span_trace = self
                .span_trace
                .as_ref()
                .or_else(|| get_deepest_spantrace(error))
                .expect("SpanTrace capture failed");

            match span_trace.status() {
                SpanTraceStatus::CAPTURED => {
                    write!(f, "\n\n")?;
                    write!(indented(f).with_format(Format::Uniform { indentation: "  " }), "{}", color_spantrace::colorize(span_trace))?
                },
                SpanTraceStatus::UNSUPPORTED => write!(f, "\n\nWarning: SpanTrace capture is Unsupported.\nEnsure that you've setup an error layer and the versions match")?,
                _ => (),
            }
        }

        if let Some(backtrace) = self.backtrace.as_ref() {
            write!(f, "\n\n")?;

            let bt_str = CONFIG
                .get_or_init(default_printer)
                .format_trace_to_string(&backtrace)
                .unwrap();

            write!(
                indented(f).with_format(Format::Uniform { indentation: "  " }),
                "{}",
                bt_str
            )?;
        } else if !self.help.is_empty() {
            writeln!(f)?;
        }

        for help in &self.help {
            write!(f, "\n{}", help)?;
        }

        Ok(())
    }
}

impl fmt::Display for InstallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("could not install the BacktracePrinter as another was already installed")
    }
}

impl std::error::Error for InstallError {}

fn backtrace_enabled() -> bool {
    // Cache the result of reading the environment variables to make
    // backtrace captures speedy, because otherwise reading environment
    // variables every time can be somewhat slow.
    static ENABLED: AtomicUsize = AtomicUsize::new(0);
    match ENABLED.load(SeqCst) {
        0 => {}
        1 => return false,
        _ => return true,
    }
    let enabled = match env::var("RUST_LIB_BACKTRACE") {
        Ok(s) => s != "0",
        Err(_) => match env::var("RUST_BACKTRACE") {
            Ok(s) => s != "0",
            Err(_) => false,
        },
    };
    ENABLED.store(enabled as usize + 1, SeqCst);
    enabled
}

#[cfg(feature = "capture-spantrace")]
fn get_deepest_spantrace<'a>(error: &'a (dyn Error + 'static)) -> Option<&'a SpanTrace> {
    Chain::new(error)
        .rev()
        .flat_map(|error| error.span_trace())
        .next()
}

/// Override the global BacktracePrinter used by `color_eyre::Context` when printing captured
/// backtraces.
///
/// # Examples
///
/// This enables configuration like custom frame filters:
///
/// ```rust
/// use color_eyre::BacktracePrinter;
///
/// let printer = BacktracePrinter::new()
///     .add_frame_filter(Box::new(|frames| {
///         let filters = &[
///             "evil_function",
///         ];
///
///         frames.retain(|frame| {
///             !filters.iter().any(|f| {
///                 let name = if let Some(name) = frame.name.as_ref() {
///                     name.as_str()
///                 } else {
///                     return true;
///                 };
///
///                 name.starts_with(f)
///             })
///         });
///     }));
///
/// color_eyre::install(printer).unwrap();
/// ```
pub fn install(printer: BacktracePrinter) -> Result<(), impl std::error::Error> {
    let printer = add_eyre_filters(printer);
    CONFIG.set(printer).map_err(|_| InstallError)
}

fn default_printer() -> BacktracePrinter {
    add_eyre_filters(BacktracePrinter::new())
}

fn add_eyre_filters(printer: BacktracePrinter) -> BacktracePrinter {
    printer.add_frame_filter(Box::new(|frames| {
        let filters = &[
            "<color_eyre::Context as eyre::EyreContext>::default",
            "eyre::",
            "color_eyre::",
        ];

        frames.retain(|frame| {
            !filters.iter().any(|f| {
                let name = if let Some(name) = frame.name.as_ref() {
                    name.as_str()
                } else {
                    return true;
                };

                name.starts_with(f)
            })
        });
    }))
}

/// A type alias for `eyre::Report<color_eyre::Context>`
///
/// # Example
///
/// ```rust
/// use color_eyre::Report;
///
/// # struct Config;
/// fn try_thing(path: &str) -> Result<Config, Report> {
///     // ...
/// # Ok(Config)
/// }
/// ```
pub type Report = eyre::Report<Context>;

/// A type alias for `Result<T, color_eyre::Report>`
///
/// # Example
///
///```
/// #[tracing::instrument]
/// fn main() -> color_eyre::Result<()> {
///
///     // ...
///
///     Ok(())
/// }
/// ```
pub type Result<T, E = Report> = core::result::Result<T, E>;
