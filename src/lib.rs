//! A custom context for the [`eyre`] crate for colorful error reports with suggestions, custom
//! sections, [`tracing-error`] support, and backtraces on stable.
//!
//! ## TLDR
//!
//! `color_eyre` helps you build error reports that look like this:
//!
//! ![custom section example](https://raw.githubusercontent.com/yaahc/color-eyre/master/pictures/custom_section.png)
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
//! ### Improving perf on debug builds
//!
//! In debug mode `color-eyre` behaves noticably worse than `eyre`. This is caused
//! by the fact that `eyre` uses `std::backtrace::Backtrace` instead of
//! `backtrace::Backtrace`. The std version of backtrace is precompiled with
//! optimizations, this means that whether or not you're in debug mode doesn't
//! matter much for how expensive backtrace capture is, it will always be in the
//! 10s of milliseconds to capture. A debug version of `backtrace::Backtrace`
//! however isn't so lucky, and can take an order of magnitude more time to capture
//! a backtrace compared to it's std counterpart.
//!
//! Cargo [profile
//! overrides](https://doc.rust-lang.org/cargo/reference/profiles.html#overrides)
//! can be used to mitigate this problem. By configuring your project to always
//! build `backtrace` with optimizations you should get the same performance from
//! `color-eyre` that you're used to with `eyre`. To do so add the following to
//! your Cargo.toml:
//!
//! ```toml
//! [profile.dev.package.backtrace]
//! opt-level = 3
//! ```
//!
//! ## Features
//!
//! ### Multiple report format verbosity levels
//!
//! `color-eyre` provides 3 different report formats for how it formats the captured `SpanTrace`
//! and `Backtrace`, minimal, short, and full. Take the following example, taken from
//! [`examples/usage.rs`]:
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
//! ---
//!
//! Running `cargo run --example usage` without `RUST_LIB_BACKTRACE` set will produce a minimal
//! report like this:
//!
//! ![minimal report format](https://raw.githubusercontent.com/yaahc/color-eyre/master/pictures/minimal.png)
//!
//! <br>
//!
//! Running `RUST_LIB_BACKTRACE=1 cargo run --example usage` tells `color-eyre` to use the short
//! format, which additionally capture a [`backtrace::Backtrace`]:
//!
//! ![short report format](https://raw.githubusercontent.com/yaahc/color-eyre/master/pictures/short.png)
//!
//! <br>
//!
//! Finally, running `RUST_LIB_BACKTRACE=full cargo run --example usage` tells `color-eyre` to use
//! the full format, which in addition to the above will attempt to include source lines where the
//! error originated from, assuming it can find them on the disk.
//!
//! ![full report format](https://raw.githubusercontent.com/yaahc/color-eyre/master/pictures/full.png)
//!
//! ### Custom `Section`s for error reports via [`Help`] trait
//!
//! The `section` module provides helpers for adding extra sections to error reports. Sections are
//! disinct from error messages and are displayed independently from the chain of errors. Take this
//! example of adding sections to contain `stderr` and `stdout` from a failed command, taken from
//! [`examples/custom_section.rs`]:
//!
//! ```rust
//! use color_eyre::{SectionExt, Help, Report};
//! use eyre::eyre;
//! use std::process::Command;
//! use tracing::instrument;
//!
//! trait Output {
//!     fn output2(&mut self) -> Result<String, Report>;
//! }
//!
//! impl Output for Command {
//!     #[instrument]
//!     fn output2(&mut self) -> Result<String, Report> {
//!         let output = self.output()?;
//!
//!         let stdout = String::from_utf8_lossy(&output.stdout);
//!
//!         if !output.status.success() {
//!             let stderr = String::from_utf8_lossy(&output.stderr);
//!             Err(eyre!("cmd exited with non-zero status code"))
//!                 .with_section(move || {
//!                     "Stdout:"
//!                         .skip_if(|| stdout.is_empty())
//!                         .body(stdout.trim().to_string())
//!                 })
//!                 .with_section(move || {
//!                     "Stderr:"
//!                         .skip_if(|| stderr.is_empty())
//!                         .body(stderr.trim().to_string())
//!                 })
//!         } else {
//!             Ok(stdout.into())
//!         }
//!     }
//! }
//! ```
//!
//! ---
//!
//! Here we have an function that, if the command exits unsuccessfully, creates a report indicating
//! the failure and attaches two sections, one for `stdout` and one for `stderr`. Each section
//! includes a short header and a body that contains the actual output. Additionally these sections
//! use `skip_if` to tell the report not to include them if there was no output, preventing empty
//! sections from polluting the end report.
//!
//! Running `cargo run --example custom_section` shows us how these sections are included in the
//! output:
//!
//! ![custom section example](https://raw.githubusercontent.com/yaahc/color-eyre/master/pictures/custom_section.png)
//!
//! Only the `Stderr:` section actually gets included. The `cat` command fails, so stdout ends up
//! being empty and is skipped in the final report. This gives us a short and concise error report
//! indicating exactly what was attempted and how it failed.
//!
//! ### Aggregating multiple errors into one report
//!
//! It's not uncommon for programs like batched task runners or parsers to want to
//! return an error with multiple sources. The current version of the error trait
//! does not support this use case very well, though there is [work being
//! done](https://github.com/rust-lang/rfcs/pull/2895) to improve this.
//!
//! For now however one way to work around this is to compose errors outside the
//! error trait. `color-eyre` supports such composition in its error reports via
//! the `Help` trait.
//!
//! For an example of how to aggregate errors check out [`examples/multiple_errors.rs`].
//!
//! ### Custom configuration for `color-backtrace` for setting custom filters and more
//!
//! The pretty printing for backtraces and span traces isn't actually provided by `color-eyre`, but
//! instead comes from its dependencies [`color-backtrace`] and [`color-spantrace`].
//! `color-backtrace` in particular has many more features than are exported by `color-eyre`, such
//! as customized color schemes, panic hooks, and custom frame filters. The custom frame filters
//! are particularly useful when combined with `color-eyre`, so to enable their usage we provide
//! the `install` fn for setting up a custom `BacktracePrinter` with custom filters installed.
//!
//! For an example of how to setup custom filters, check out [`examples/custom_filter.rs`].
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
//! [`eyre`]: https://docs.rs/eyre
//! [`tracing-error`]: https://docs.rs/tracing-error
//! [`color-backtrace`]: https://docs.rs/color-backtrace
//! [`eyre::EyreContext`]: https://docs.rs/eyre/*/eyre/trait.EyreContext.html
//! [`backtrace::Backtrace`]: https://docs.rs/backtrace/*/backtrace/struct.Backtrace.html
//! [`tracing_error::SpanTrace`]: https://docs.rs/tracing-error/*/tracing_error/struct.SpanTrace.html
//! [`color-spantrace`]: https://github.com/yaahc/color-spantrace
//! [`Help`]: https://docs.rs/color-eyre/*/color_eyre/trait.Help.html
//! [`eyre::Report`]: https://docs.rs/eyre/*/eyre/struct.Report.html
//! [`eyre::Result`]: https://docs.rs/eyre/*/eyre/type.Result.html
//! [`Context`]: https://docs.rs/color-eyre/*/color_eyre/struct.Context.html
//! [`examples/usage.rs`]: https://github.com/yaahc/color-eyre/blob/master/examples/usage.rs
//! [`examples/custom_filter.rs`]: https://github.com/yaahc/color-eyre/blob/master/examples/custom_filter.rs
//! [`examples/custom_section.rs`]: https://github.com/yaahc/color-eyre/blob/master/examples/custom_section.rs
//! [`examples/multiple_errors.rs`]: https://github.com/yaahc/color-eyre/blob/master/examples/multiple_errors.rs
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
use indenter::{indented, Format};
use once_cell::sync::OnceCell;
use section::Order;
pub use section::{help::Help, Section, SectionExt};
#[cfg(feature = "capture-spantrace")]
use std::error::Error;
use std::{
    env,
    fmt::{self, Write as _},
    sync::atomic::{AtomicUsize, Ordering::SeqCst},
};
#[cfg(feature = "capture-spantrace")]
use tracing_error::{ExtractSpanTrace, SpanTrace, SpanTraceStatus};

pub mod section;

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
    sections: Vec<Section>,
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
            sections: Vec::new(),
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

        for section in self
            .sections
            .iter()
            .filter(|s| matches!(s.order, Order::AfterErrMsgs))
        {
            write!(f, "\n\n{:?}", section)?;
        }

        for section in self
            .sections
            .iter()
            .filter(|s| matches!(s.order, Order::BeforeSpanTrace))
        {
            write!(f, "\n\n{:?}", section)?;
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
        } else if self
            .sections
            .iter()
            .any(|s| matches!(s.order, Order::AfterBackTrace))
        {
            writeln!(f)?;
        }

        for section in self
            .sections
            .iter()
            .filter(|s| matches!(s.order, Order::AfterBackTrace))
        {
            write!(f, "\n{:?}", section)?;
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
