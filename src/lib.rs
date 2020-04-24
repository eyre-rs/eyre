mod help;

pub use eyre::*;

pub use help::Help;
pub type Report = eyre::Report<JaneContext>;
pub type Result<T, E = Report> = core::result::Result<T, E>;

use backtrace::Backtrace;
use help::HelpInfo;
use indenter::Indented;
use std::any::{Any, TypeId};
use std::error::Error;
use std::fmt::Write as _;
use tracing_error::{ExtractSpanTrace, SpanTrace, SpanTraceStatus};

pub struct JaneContext {
    backtrace: Backtrace,
    span_trace: Option<SpanTrace>,
    help: Vec<HelpInfo>,
}

fn get_deepest_spantrace<'a>(error: &'a (dyn Error + 'static)) -> Option<&'a SpanTrace> {
    Chain::new(error)
        .rev()
        .flat_map(|error| error.span_trace())
        .next()
}

impl EyreContext for JaneContext {
    fn default(error: &(dyn std::error::Error + 'static)) -> Self {
        let backtrace = Backtrace::new();

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
            .rev()
            .filter(|e| e.span_trace().is_none())
            .enumerate();

        for (n, error) in errors {
            writeln!(f)?;
            write!(Indented::numbered(f, n), "{}", error)?;
        }

        for help in &self.help {
            write!(f, "\n{}", help)?;
        }

        let span_trace = self
            .span_trace
            .as_ref()
            .or_else(|| get_deepest_spantrace(error))
            .expect("SpanTrace capture failed");

        match span_trace.status() {
            SpanTraceStatus::CAPTURED => write!(f, "\n\nSpan Trace:\n{}", span_trace)?,
            SpanTraceStatus::UNSUPPORTED => write!(f, "\n\nWarning: SpanTrace capture is Unsupported.\nEnsure that you've setup an error layer and the versions match")?,
            _ => (),
        }

        let backtrace = &self.backtrace;

        write!(f, "\n\n")?;
        let settings = color_backtrace::Settings::default();
        write!(
            f,
            "{}",
            color_backtrace::print_backtrace(&backtrace, &settings)
        )?;

        Ok(())
    }
}
