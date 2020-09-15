use crate::config::installed_hook;
use crate::{
    section::help::HelpInfo,
    writers::{EnvSection, WriterExt},
    Handler,
};
use backtrace::Backtrace;
use indenter::{indented, Format};
use owo_colors::OwoColorize;
use std::fmt::Write;
#[cfg(feature = "capture-spantrace")]
use tracing_error::{ExtractSpanTrace, SpanTrace};

impl Handler {
    /// Return a reference to the captured `Backtrace` type
    pub fn backtrace(&self) -> Option<&Backtrace> {
        self.backtrace.as_ref()
    }

    /// Return a reference to the captured `SpanTrace` type
    #[cfg(feature = "capture-spantrace")]
    #[cfg_attr(docsrs, doc(cfg(feature = "capture-spantrace")))]
    pub fn span_trace(&self) -> Option<&SpanTrace> {
        self.span_trace.as_ref()
    }
}

impl Handler {}

impl eyre::EyreHandler for Handler {
    fn debug(
        &self,
        error: &(dyn std::error::Error + 'static),
        f: &mut core::fmt::Formatter<'_>,
    ) -> core::fmt::Result {
        if f.alternate() {
            return core::fmt::Debug::fmt(error, f);
        }

        #[cfg(feature = "capture-spantrace")]
        let errors = eyre::Chain::new(error)
            .filter(|e| e.span_trace().is_none())
            .enumerate();

        #[cfg(not(feature = "capture-spantrace"))]
        let errors = eyre::Chain::new(error).enumerate();

        for (n, error) in errors {
            writeln!(f)?;
            write!(indented(f).ind(n), "{}", error.bright_red())?;
        }

        let mut separated = f.header("\n\n");

        for section in self
            .sections
            .iter()
            .filter(|s| matches!(s, HelpInfo::Error(_)))
        {
            write!(separated.ready(), "{}", section)?;
        }

        for section in self
            .sections
            .iter()
            .filter(|s| matches!(s, HelpInfo::Custom(_)))
        {
            write!(separated.ready(), "{}", section)?;
        }

        #[cfg(feature = "capture-spantrace")]
        let span_trace = self
            .span_trace
            .as_ref()
            .or_else(|| get_deepest_spantrace(error));

        #[cfg(feature = "capture-spantrace")]
        {
            if let Some(span_trace) = span_trace {
                write!(
                    &mut separated.ready(),
                    "{}",
                    crate::writers::FormattedSpanTrace(span_trace)
                )?;
            }
        }

        if let Some(backtrace) = self.backtrace.as_ref() {
            let fmted_bt = installed_hook().format_backtrace(&backtrace);

            write!(
                indented(&mut separated.ready()).with_format(Format::Uniform { indentation: "  " }),
                "{}",
                fmted_bt
            )?;
        }

        let f = separated.ready();
        let mut h = f.header("\n");
        let mut f = h.in_progress();

        for section in self
            .sections
            .iter()
            .filter(|s| !matches!(s, HelpInfo::Custom(_) | HelpInfo::Error(_)))
        {
            write!(&mut f, "{}", section)?;
            f = h.ready();
        }

        if self.display_env_section {
            let env_section = EnvSection {
                bt_captured: &self.backtrace.is_some(),
                #[cfg(feature = "capture-spantrace")]
                span_trace,
            };

            write!(&mut separated.ready(), "{}", env_section)?;
        }

        Ok(())
    }
}

#[cfg(feature = "capture-spantrace")]
pub(crate) fn get_deepest_spantrace<'a>(
    error: &'a (dyn std::error::Error + 'static),
) -> Option<&'a SpanTrace> {
    eyre::Chain::new(error)
        .rev()
        .flat_map(|error| error.span_trace())
        .next()
}
