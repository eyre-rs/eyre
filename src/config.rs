//! Configuration options for customizing the behavior of the provided panic
//! and error reporting hooks
use crate::ColorExt;
use ansi_term::Color::*;
use std::env;
use std::{fmt, path::PathBuf, sync::Arc};

#[derive(Debug)]
struct InstallError;

impl fmt::Display for InstallError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.write_str("could not install the BacktracePrinter as another was already installed")
    }
}

impl std::error::Error for InstallError {}

/// A representation of a Frame from a Backtrace or a SpanTrace
#[derive(Debug)]
#[non_exhaustive]
pub struct Frame {
    /// Frame index
    pub n: usize,
    /// frame symbol name
    pub name: Option<String>,
    /// source line number
    pub lineno: Option<u32>,
    /// source file path
    pub filename: Option<PathBuf>,
}

impl fmt::Display for Frame {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let is_dependency_code = self.is_dependency_code();

        // Print frame index.
        write!(f, "{:>2}: ", self.n)?;

        // Does the function have a hash suffix?
        // (dodging a dep on the regex crate here)
        let name = self.name.as_deref().unwrap_or("<unknown>");
        let has_hash_suffix = name.len() > 19
            && &name[name.len() - 19..name.len() - 16] == "::h"
            && name[name.len() - 16..].chars().all(|x| x.is_digit(16));

        // Print function name.

        if has_hash_suffix {
            let color = if is_dependency_code {
                Green
            } else {
                Red.make_intense()
            };
            write!(f, "{}", color.paint(&name[..name.len() - 19]))?;
            let color = Black.make_intense();
            writeln!(f, "{}", color.paint(&name[name.len() - 19..]))?;
        } else {
            writeln!(f, "{}", name)?;
        }

        // Print source location, if known.
        if let Some(ref file) = self.filename {
            let filestr = file.to_str().unwrap_or("<bad utf8>");
            let lineno = self
                .lineno
                .map_or("<unknown line>".to_owned(), |x| x.to_string());
            writeln!(
                f,
                "    at {}:{}",
                Purple.paint(filestr),
                Purple.paint(lineno)
            )?;
        } else {
            writeln!(f, "    at <unknown source file>")?;
        }

        let v = if std::thread::panicking() {
            panic_verbosity()
        } else {
            lib_verbosity()
        };

        // Maybe print source.
        if v >= Verbosity::Full {
            self.print_source_if_avail(f)?;
        }

        Ok(())
    }
}

impl Frame {
    fn is_dependency_code(&self) -> bool {
        const SYM_PREFIXES: &[&str] = &[
            "std::",
            "core::",
            "backtrace::backtrace::",
            "_rust_begin_unwind",
            "color_traceback::",
            "__rust_",
            "___rust_",
            "__pthread",
            "_main",
            "main",
            "__scrt_common_main_seh",
            "BaseThreadInitThunk",
            "_start",
            "__libc_start_main",
            "start_thread",
        ];

        // Inspect name.
        if let Some(ref name) = self.name {
            if SYM_PREFIXES.iter().any(|x| name.starts_with(x)) {
                return true;
            }
        }

        const FILE_PREFIXES: &[&str] = &[
            "/rustc/",
            "src/libstd/",
            "src/libpanic_unwind/",
            "src/libtest/",
        ];

        // Inspect filename.
        if let Some(ref filename) = self.filename {
            let filename = filename.to_string_lossy();
            if FILE_PREFIXES.iter().any(|x| filename.starts_with(x))
                || filename.contains("/.cargo/registry/src/")
            {
                return true;
            }
        }

        false
    }

    /// Heuristically determine whether a frame is likely to be a post panic
    /// frame.
    ///
    /// Post panic frames are frames of a functions called after the actual panic
    /// is already in progress and don't contain any useful information for a
    /// reader of the backtrace.
    fn is_post_panic_code(&self) -> bool {
        const SYM_PREFIXES: &[&str] = &[
            "_rust_begin_unwind",
            "rust_begin_unwind",
            "core::result::unwrap_failed",
            "core::option::expect_none_failed",
            "core::panicking::panic_fmt",
            "color_backtrace::create_panic_handler",
            "std::panicking::begin_panic",
            "begin_panic_fmt",
            "failure::backtrace::Backtrace::new",
            "backtrace::capture",
            "failure::error_message::err_msg",
            "<failure::error::Error as core::convert::From<F>>::from",
        ];

        match self.name.as_ref() {
            Some(name) => SYM_PREFIXES.iter().any(|x| name.starts_with(x)),
            None => false,
        }
    }

    /// Heuristically determine whether a frame is likely to be part of language
    /// runtime.
    fn is_runtime_init_code(&self) -> bool {
        const SYM_PREFIXES: &[&str] =
            &["std::rt::lang_start::", "test::run_test::run_test_inner::"];

        let (name, file) = match (self.name.as_ref(), self.filename.as_ref()) {
            (Some(name), Some(filename)) => (name, filename.to_string_lossy()),
            _ => return false,
        };

        if SYM_PREFIXES.iter().any(|x| name.starts_with(x)) {
            return true;
        }

        // For Linux, this is the best rule for skipping test init I found.
        if name == "{{closure}}" && file == "src/libtest/lib.rs" {
            return true;
        }

        false
    }

    fn print_source_if_avail(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let (lineno, filename) = match (self.lineno, self.filename.as_ref()) {
            (Some(a), Some(b)) => (a, b),
            // Without a line number and file name, we can't sensibly proceed.
            _ => return Ok(()),
        };

        let file = match std::fs::File::open(filename) {
            Ok(file) => file,
            Err(ref e) if e.kind() == std::io::ErrorKind::NotFound => return Ok(()),
            e @ Err(_) => e.unwrap(),
        };

        use std::fmt::Write;
        use std::io::BufRead;

        // Extract relevant lines.
        let reader = std::io::BufReader::new(file);
        let start_line = lineno - 2.min(lineno - 1);
        let surrounding_src = reader.lines().skip(start_line as usize - 1).take(5);
        let mut buf = String::new();
        for (line, cur_line_no) in surrounding_src.zip(start_line..) {
            let line = line.unwrap();
            if cur_line_no == lineno {
                let color = White.bold();
                write!(&mut buf, "{:>8} > {}", cur_line_no, line)?;
                writeln!(f, "{}", color.paint(&buf))?;
                buf.clear();
            } else {
                writeln!(f, "{:>8} │ {}", cur_line_no, line)?;
            }
        }

        Ok(())
    }
}

/// Builder for customizing the behavior of the global panic and error report hooks
pub struct HookBuilder {
    filters: Vec<Box<FilterCallback>>,
    capture_span_trace_by_default: bool,
}

impl HookBuilder {
    /// Construct a HookBuilder
    ///
    /// # Details
    ///
    /// By default this function calls `add_default_filters()` and
    /// `capture_span_trace_by_default(true)`. To get a `HookBuilder` with all
    /// features disabled by default call `HookBuilder::blank()`.
    ///
    /// # Example
    ///
    /// ```rust
    /// use color_eyre::config::HookBuilder;
    ///
    /// HookBuilder::new()
    ///     .install()
    ///     .unwrap();
    /// ```
    pub fn new() -> Self {
        Self::blank()
            .add_default_filters()
            .capture_span_trace_by_default(true)
    }

    /// Construct a HookBuilder with minimal features enabled
    pub fn blank() -> Self {
        HookBuilder {
            filters: vec![],
            capture_span_trace_by_default: false,
        }
    }

    /// Configures the default capture mode for `SpanTraces` in error reports and panics
    pub fn capture_span_trace_by_default(mut self, cond: bool) -> Self {
        self.capture_span_trace_by_default = cond;
        self
    }

    /// Add a custom filter to the set of frame filters
    ///
    /// # Examples
    ///
    /// ```rust
    /// color_eyre::config::HookBuilder::default()
    ///     .add_frame_filter(Box::new(|frames| {
    ///         let filters = &[
    ///             "uninteresting_function",
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
    ///     }))
    ///     .install()
    ///     .unwrap();
    /// ```
    pub fn add_frame_filter(mut self, filter: Box<FilterCallback>) -> Self {
        self.filters.push(filter);
        self
    }

    /// Install the given Hook as the global error report hook
    pub fn install(self) -> Result<(), crate::eyre::Report> {
        let (panic_hook, eyre_hook) = self.into_hooks();
        crate::eyre::set_hook(Box::new(move |e| Box::new(eyre_hook.default(e))))?;
        install_panic_hook();

        if crate::CONFIG.set(panic_hook).is_err() {
            Err(InstallError)?
        }

        Ok(())
    }

    /// Add the default set of filters to this `HookBuilder`'s configuration
    pub fn add_default_filters(self) -> Self {
        self.add_frame_filter(Box::new(default_frame_filter))
            .add_frame_filter(Box::new(eyre_frame_filters))
    }

    pub(crate) fn into_hooks(self) -> (PanicHook, EyreHook) {
        let panic_hook = PanicHook {
            filters: self.filters.into_iter().map(Into::into).collect(),
            capture_span_trace_by_default: self.capture_span_trace_by_default,
        };

        let eyre_hook = EyreHook {
            capture_span_trace_by_default: self.capture_span_trace_by_default,
        };

        (panic_hook, eyre_hook)
    }
}

impl Default for HookBuilder {
    fn default() -> Self {
        Self::new()
    }
}

fn default_frame_filter(frames: &mut Vec<&Frame>) {
    let top_cutoff = frames
        .iter()
        .rposition(|x| x.is_post_panic_code())
        .map(|x| x + 2) // indices are 1 based
        .unwrap_or(0);

    let bottom_cutoff = frames
        .iter()
        .position(|x| x.is_runtime_init_code())
        .unwrap_or_else(|| frames.len());

    let rng = top_cutoff..=bottom_cutoff;
    frames.retain(|x| rng.contains(&x.n))
}

fn eyre_frame_filters(frames: &mut Vec<&Frame>) {
    let filters = &[
        "<color_eyre::Handler as eyre::EyreHandler>::default",
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
}

struct PanicPrinter<'a>(&'a std::panic::PanicInfo<'a>);

impl fmt::Display for PanicPrinter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        print_panic_info(self, f)
    }
}

fn install_panic_hook() {
    std::panic::set_hook(Box::new(|pi| eprintln!("{}", PanicPrinter(pi))))
}

fn print_panic_info(printer: &PanicPrinter<'_>, out: &mut fmt::Formatter<'_>) -> fmt::Result {
    let pi = printer.0;

    writeln!(out, "{}", Red.paint("The application panicked (crashed)."))?;

    // Print panic message.
    let payload = pi
        .payload()
        .downcast_ref::<String>()
        .map(String::as_str)
        .or_else(|| pi.payload().downcast_ref::<&str>().cloned())
        .unwrap_or("<non string panic payload>");

    write!(out, "Message:  ")?;
    writeln!(out, "{}", Cyan.paint(payload))?;

    // If known, print panic location.
    write!(out, "Location: ")?;
    if let Some(loc) = pi.location() {
        write!(out, "{}", Purple.paint(loc.file()))?;
        write!(out, ":")?;
        writeln!(out, "{}", Purple.paint(loc.line().to_string()))?;
    } else {
        writeln!(out, "<unknown>")?;
    }

    let v = panic_verbosity();

    // Print some info on how to increase verbosity.
    if v == Verbosity::Minimal {
        write!(out, "\nBacktrace omitted.\n\nRun with ")?;
        write!(out, "RUST_BACKTRACE=1")?;
        writeln!(out, " environment variable to display it.")?;
    } else {
        // This text only makes sense if frames are displayed.
        write!(out, "\nRun with ")?;
        write!(out, "COLORBT_SHOW_HIDDEN=1")?;
        writeln!(out, " environment variable to disable frame filtering.")?;
    }
    if v <= Verbosity::Medium {
        write!(out, "Run with ")?;
        write!(out, "RUST_BACKTRACE=full")?;
        writeln!(out, " to include source snippets.")?;
    }

    let printer = installed_printer();

    #[cfg(feature = "capture-spantrace")]
    {
        if printer.spantrace_capture_enabled() {
            let span_trace = tracing_error::SpanTrace::capture();
            write!(out, "{}", crate::writers::FormattedSpanTrace(&span_trace))?;
        }
    }

    if panic_verbosity() != Verbosity::Minimal {
        let bt = backtrace::Backtrace::new();
        let fmt_bt = printer.format_backtrace(&bt);
        writeln!(out, "\n\n{}", fmt_bt)?;
    }

    Ok(())
}

pub(crate) struct PanicHook {
    filters: Vec<Arc<FilterCallback>>,
    capture_span_trace_by_default: bool,
}

impl PanicHook {
    pub(crate) fn format_backtrace<'a>(
        &'a self,
        trace: &'a backtrace::Backtrace,
    ) -> BacktraceFormatter<'a> {
        BacktraceFormatter {
            printer: self,
            inner: trace,
        }
    }

    fn spantrace_capture_enabled(&self) -> bool {
        std::env::var("RUST_SPANTRACE")
            .map(|val| val != "0")
            .unwrap_or(self.capture_span_trace_by_default)
    }
}

pub(crate) struct EyreHook {
    capture_span_trace_by_default: bool,
}

impl EyreHook {
    #[allow(unused_variables)]
    pub(crate) fn default(&self, error: &(dyn std::error::Error + 'static)) -> crate::Handler {
        let backtrace = if lib_verbosity() != Verbosity::Minimal {
            Some(backtrace::Backtrace::new())
        } else {
            None
        };

        #[cfg(feature = "capture-spantrace")]
        let span_trace = if self.spantrace_capture_enabled()
            && crate::handler::get_deepest_spantrace(error).is_none()
        {
            Some(tracing_error::SpanTrace::capture())
        } else {
            None
        };

        crate::Handler {
            backtrace,
            #[cfg(feature = "capture-spantrace")]
            span_trace,
            sections: Vec::new(),
        }
    }

    fn spantrace_capture_enabled(&self) -> bool {
        std::env::var("RUST_SPANTRACE")
            .map(|val| val != "0")
            .unwrap_or(self.capture_span_trace_by_default)
    }
}

pub(crate) struct BacktraceFormatter<'a> {
    printer: &'a PanicHook,
    inner: &'a backtrace::Backtrace,
}

impl fmt::Display for BacktraceFormatter<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        writeln!(f, "{:━^80}", " BACKTRACE ")?;

        // Collect frame info.
        let frames: Vec<_> = self
            .inner
            .frames()
            .iter()
            .flat_map(|frame| frame.symbols())
            .zip(1usize..)
            .map(|(sym, n)| Frame {
                name: sym.name().map(|x| x.to_string()),
                lineno: sym.lineno(),
                filename: sym.filename().map(|x| x.into()),
                n,
            })
            .collect();

        let mut filtered_frames = frames.iter().collect();
        match env::var("COLORBT_SHOW_HIDDEN").ok().as_deref() {
            Some("1") | Some("on") | Some("y") => (),
            _ => {
                for filter in &self.printer.filters {
                    filter(&mut filtered_frames);
                }
            }
        }

        if filtered_frames.is_empty() {
            // TODO: Would probably look better centered.
            return writeln!(f, "<empty backtrace>");
        }

        // Don't let filters mess with the order.
        filtered_frames.sort_by_key(|x| x.n);

        macro_rules! print_hidden {
            ($n:expr) => {
                let color = Cyan.make_intense();
                let n = $n;
                let text = format!(
                    "{:^80}",
                    format!(
                        "{decorator} {n} frame{plural} hidden {decorator}",
                        n = n,
                        plural = if n == 1 { "" } else { "s" },
                        decorator = "⋮",
                    )
                );
                writeln!(f, "{}", color.paint(text))?;
            };
        }

        let mut last_n = 0;
        for frame in &filtered_frames {
            let frame_delta = frame.n - last_n - 1;
            if frame_delta != 0 {
                print_hidden!(frame_delta);
            }
            write!(f, "{}", frame)?;
            last_n = frame.n;
        }

        let last_filtered_n = filtered_frames.last().unwrap().n;
        let last_unfiltered_n = frames.last().unwrap().n;
        if last_filtered_n < last_unfiltered_n {
            print_hidden!(last_unfiltered_n - last_filtered_n);
        }

        Ok(())
    }
}

pub(crate) fn installed_printer() -> &'static PanicHook {
    crate::CONFIG.get_or_init(default_printer)
}

fn default_printer() -> PanicHook {
    let (panic_hook, _) = HookBuilder::default().into_hooks();
    panic_hook
}

#[derive(PartialEq, Eq, PartialOrd, Ord, Clone, Copy)]
pub(crate) enum Verbosity {
    Minimal,
    Medium,
    Full,
}

fn panic_verbosity() -> Verbosity {
    match env::var("RUST_BACKTRACE") {
        Ok(s) if s == "full" => Verbosity::Full,
        Ok(s) if s != "0" => Verbosity::Medium,
        _ => Verbosity::Minimal,
    }
}

pub(crate) fn lib_verbosity() -> Verbosity {
    match env::var("RUST_LIB_BACKTRACE").or_else(|_| env::var("RUST_BACKTRACE")) {
        Ok(s) if s == "full" => Verbosity::Full,
        Ok(s) if s != "0" => Verbosity::Medium,
        _ => Verbosity::Minimal,
    }
}

/// Callback for filtering a vector of `Frame`s
pub type FilterCallback = dyn Fn(&mut Vec<&Frame>) + Send + Sync + 'static;
