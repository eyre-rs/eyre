//! A rust library for colorizing [`tracing_error::SpanTrace`] objects in the style
//! of [`color-backtrace`].
//!
//! ## Setup
//!
//! Add the following to your `Cargo.toml`:
//!
//! ```toml
//! [dependencies]
//! color-spantrace = "0.1"
//! tracing = "0.1.13"
//! tracing-error = "0.1.2"
//! tracing-subscriber = "0.2.5"
//! ```
//!
//! Setup a tracing subscriber with an `ErrorLayer`:
//!
//! ```rust
//! use tracing_error::ErrorLayer;
//! use tracing_subscriber::{prelude::*, registry::Registry};
//!
//! Registry::default().with(ErrorLayer::default()).init();
//! ```
//!
//! Create spans and enter them:
//!
//! ```rust
//! use tracing::instrument;
//! use tracing_error::SpanTrace;
//!
//! #[instrument]
//! fn foo() -> SpanTrace {
//!     SpanTrace::capture()
//! }
//! ```
//!
//! And finally colorize the `SpanTrace`:
//!
//! ```rust
//! use tracing_error::SpanTrace;
//!
//! let span_trace = SpanTrace::capture();
//! println!("{}", color_spantrace::colorize(&span_trace));
//! ```
//!
//! [`tracing_error::SpanTrace`]: https://docs.rs/tracing-error/*/tracing_error/struct.SpanTrace.html
//! [`color-backtrace`]: https://github.com/athre0z/color-backtrace
#![doc(html_root_url = "https://docs.rs/color-spantrace/0.1.2")]
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
use ansi_term::{
    Color::{Cyan, Red},
    Style,
};
use std::env;
use std::fmt;
use std::fs::File;
use std::io::{BufRead, BufReader, ErrorKind};
use tracing_error::SpanTrace;

/// Display a [`SpanTrace`] with colors and source
///
/// This function returns an `impl Display` type which can be then used in place of the original
/// SpanTrace when writing it too the screen or buffer.
///
/// # Example
///
/// ```rust
/// use tracing_error::SpanTrace;
///
/// let span_trace = SpanTrace::capture();
/// println!("{}", color_spantrace::colorize(&span_trace));
/// ```
///
/// [`SpanTrace`]: https://docs.rs/tracing-error/*/tracing_error/struct.SpanTrace.html
pub fn colorize(span_trace: &SpanTrace) -> impl fmt::Display + '_ {
    ColorSpanTrace { span_trace }
}

struct ColorSpanTrace<'a> {
    span_trace: &'a SpanTrace,
}

macro_rules! try_bool {
    ($e:expr, $dest:ident) => {{
        let ret = $e.unwrap_or_else(|e| $dest = Err(e));

        if $dest.is_err() {
            return false;
        }

        ret
    }};
}

struct Frame<'a> {
    metadata: &'a tracing_core::Metadata<'static>,
    fields: &'a str,
}

/// Defines how verbose the backtrace is supposed to be.
#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord)]
enum Verbosity {
    /// Print a small message including the panic payload and the panic location.
    Minimal,
    /// Everything in `Minimal` and additionally print a backtrace.
    Medium,
    /// Everything in `Medium` plus source snippets for all backtrace locations.
    Full,
}

impl Verbosity {
    fn lib_from_env() -> Self {
        Self::convert_env(
            env::var("RUST_LIB_BACKTRACE")
                .or_else(|_| env::var("RUST_BACKTRACE"))
                .ok(),
        )
    }

    fn convert_env(env: Option<String>) -> Self {
        match env {
            Some(ref x) if x == "full" => Verbosity::Full,
            Some(_) => Verbosity::Medium,
            None => Verbosity::Minimal,
        }
    }
}

impl Frame<'_> {
    fn print(&self, i: u32, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.print_header(i, f)?;
        self.print_fields(f)?;
        self.print_source_location(f)?;
        Ok(())
    }

    fn print_header(&self, i: u32, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(
            f,
            "{:>2}: {}{}{}",
            i,
            Red.make_intense().paint(self.metadata.target()),
            Red.make_intense().paint("::"),
            Red.make_intense().paint(self.metadata.name()),
        )
    }

    fn print_fields(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if !self.fields.is_empty() {
            write!(f, " with {}", Cyan.make_intense().paint(self.fields))?;
        }

        Ok(())
    }

    fn print_source_location(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if let Some(file) = self.metadata.file() {
            let lineno = self
                .metadata
                .line()
                .map_or("<unknown line>".to_owned(), |x| x.to_string());
            write!(f, "\n    at {}:{}", file, lineno)?;
        } else {
            write!(f, "\n    at <unknown source file>")?;
        }

        Ok(())
    }

    fn print_source_if_avail(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (lineno, filename) = match (self.metadata.line(), self.metadata.file()) {
            (Some(a), Some(b)) => (a, b),
            // Without a line number and file name, we can't sensibly proceed.
            _ => return Ok(()),
        };

        let file = match File::open(filename) {
            Ok(file) => file,
            Err(ref e) if e.kind() == ErrorKind::NotFound => return Ok(()),
            e @ Err(_) => e.unwrap(),
        };

        // Extract relevant lines.
        let reader = BufReader::new(file);
        let start_line = lineno - 2.min(lineno - 1);
        let surrounding_src = reader.lines().skip(start_line as usize - 1).take(5);
        let bold = Style::new().bold();
        for (line, cur_line_no) in surrounding_src.zip(start_line..) {
            if cur_line_no == lineno {
                write!(
                    f,
                    "\n{:>8}{}{}",
                    bold.paint(cur_line_no.to_string()),
                    bold.paint(" > "),
                    bold.paint(line.unwrap())
                )?;
            } else {
                write!(f, "\n{:>8} │ {}", cur_line_no, line.unwrap())?;
            }
        }

        Ok(())
    }
}

impl fmt::Display for ColorSpanTrace<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut err = Ok(());
        let mut span = 0;

        writeln!(f, "{:━^80}\n", " SPANTRACE ")?;
        self.span_trace.with_spans(|metadata, fields| {
            let frame = Frame { metadata, fields };

            if span > 0 {
                try_bool!(write!(f, "\n",), err);
            }

            try_bool!(frame.print(span, f), err);

            if Verbosity::lib_from_env() == Verbosity::Full {
                try_bool!(frame.print_source_if_avail(f), err);
            }

            span += 1;
            true
        });

        err
    }
}

// TODO: remove when / if ansi_term merges these changes upstream
trait ColorExt {
    fn make_intense(self) -> Self;
}

impl ColorExt for ansi_term::Color {
    fn make_intense(self) -> Self {
        use ansi_term::Color::*;

        match self {
            Black => Fixed(8),
            Red => Fixed(9),
            Green => Fixed(10),
            Yellow => Fixed(11),
            Blue => Fixed(12),
            Purple => Fixed(13),
            Cyan => Fixed(14),
            White => Fixed(15),
            Fixed(color) if color < 8 => Fixed(color + 8),
            other => other,
        }
    }
}
impl ColorExt for ansi_term::Style {
    fn make_intense(mut self) -> Self {
        if let Some(color) = self.foreground {
            self.foreground = Some(color.make_intense());
        }
        self
    }
}
