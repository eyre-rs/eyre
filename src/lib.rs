//! This library provides a custom [`eyre::EyreContext`] type for colorful error
//! reports with custom help text for the [`eyre`] crate.
//!
//! **Disclaimer**: This library is currently in pre-release while I work on upstreaming changes I
//! made to [`color-backtrace`], until then this depends on unreleased versions on github and so it
//! cannot be published to crates.io
//!
//! # Features
//!
//! - captures a [`backtrace::Backtrace`] and prints using [`color-backtrace`]
//! - captures a [`tracing_error::SpanTrace`] and prints using
//! [`color-spantrace`]
//! - Only capture SpanTrace by default for better performance.
//! - display source lines when `RUST_LIB_BACKTRACE=full` is set from both of
//!   the above libraries
//! - store help text via [`Help`] trait and display after final report
//!
//! # Example
//!
//! ```should_panic
//! use color_eyre::{Help, Report};
//! use eyre::WrapErr;
//! use tracing::{info, instrument};
//! use tracing_error::ErrorLayer;
//! use tracing_subscriber::prelude::*;
//! use tracing_subscriber::{fmt, EnvFilter};
//!
//! #[instrument]
//! fn main() -> Result<(), Report> {
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
//!
//!     Ok(read_config()?)
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
//! # Report Formats
//!
//! The following report formats are available via setting the `RUST_LIB_BACKTRACE` variable.
//!
//! ## Minimal Report Format
//!
//! ![minimal report format](https://github.com/yaahc/color-eyre/blob/master/pictures/minimal.png)
//!
//! ## Short Report Format (with `RUST_LIB_BACKTRACE=1`)
//!
//! ![short report format](https://github.com/yaahc/color-eyre/blob/master/pictures/short.png)
//!
//! ## Full Report Format (with `RUST_LIB_BACKTRACE=full`)
//!
//! ![full report format](https://github.com/yaahc/color-eyre/blob/master/pictures/full.png)
//!
//! [`eyre::EyreContext`]: https://docs.rs/eyre/0.3.8/eyre/trait.EyreContext.html
//! [`eyre`]: https://docs.rs/eyre
//! [`backtrace::Backtrace`]: https://docs.rs/backtrace/0.3.46/backtrace/struct.Backtrace.html
//! [`tracing_error::SpanTrace`]: https://docs.rs/tracing-error/0.1.2/tracing_error/struct.SpanTrace.html
//! [`color-backtrace`]: https://docs.rs/color-backtrace
//! [`color-spantrace`]: https://github.com/yaahc/color-spantrace
//! [`Help`]: trait.Help.html
//! [`eyre::Report`]: https://docs.rs/eyre/0.3.8/eyre/struct.Report.html
//! [`tracing-error`]: https://docs.rs/tracing-error
use backtrace::Backtrace;
use console::style;
use eyre::*;
pub use help::Help;
use help::HelpInfo;
use indenter::Indented;
use std::error::Error;
use std::fmt::Write as _;
use tracing_error::{ExtractSpanTrace, SpanTrace, SpanTraceStatus};

mod help;

/// A Custom Context type for [`eyre::Report`] which provides colorful error
/// reports and [`tracing-error`] support.
///
/// This type is not intended to be used directly, prefer using it via the
/// [`color_eyre::Report`] and [`color_eyre::Result`] type aliases.
///
/// [`eyre::Report`]: https://docs.rs/eyre/0.3.8/eyre/struct.Report.html
/// [`tracing-error`]: https://docs.rs/tracing-error
/// [`color_eyre::Report`]: type.Report.html
/// [`color_eyre::Result`]: type.Result.html
pub struct Context {
    backtrace: Option<Backtrace>,
    span_trace: Option<SpanTrace>,
    help: Vec<HelpInfo>,
}

impl EyreContext for Context {
    fn default(error: &(dyn std::error::Error + 'static)) -> Self {
        let backtrace = if std::env::var("RUST_LIB_BACKTRACE").is_ok() {
            Some(Backtrace::new())
        } else {
            None
        };

        let span_trace = if get_deepest_spantrace(error).is_none() {
            Some(SpanTrace::capture())
        } else {
            None
        };

        Self {
            backtrace,
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

        let errors = Chain::new(error)
            .filter(|e| e.span_trace().is_none())
            .enumerate();

        for (n, error) in errors {
            writeln!(f)?;
            write!(Indented::numbered(f, n), "{}", style(error).red().dim())?;
        }

        let span_trace = self
            .span_trace
            .as_ref()
            .or_else(|| get_deepest_spantrace(error))
            .expect("SpanTrace capture failed");

        match span_trace.status() {
            SpanTraceStatus::CAPTURED => write!(f, "\n\n{}", color_spantrace::colorize(span_trace))?,
            SpanTraceStatus::UNSUPPORTED => write!(f, "\n\nWarning: SpanTrace capture is Unsupported.\nEnsure that you've setup an error layer and the versions match")?,
            _ => (),
        }

        if let Some(backtrace) = self.backtrace.as_ref() {
            write!(f, "\n\n")?;
            let settings = color_backtrace::Settings::default().add_post_panic_frames(&[
                "<color_eyre::Context as eyre::EyreContext>::default",
                "eyre::",
            ]);

            write!(
                f,
                "{}",
                color_backtrace::print_backtrace(&backtrace, &settings)
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

fn get_deepest_spantrace<'a>(error: &'a (dyn Error + 'static)) -> Option<&'a SpanTrace> {
    Chain::new(error)
        .rev()
        .flat_map(|error| error.span_trace())
        .next()
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
