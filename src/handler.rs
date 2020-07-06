use crate::config::installed_printer;
use crate::ColorExt;
use crate::{section::help::HelpInfo, writers::HeaderWriter, Handler};
use ansi_term::Color::*;
use backtrace::Backtrace;
use indenter::{indented, Format};
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

        let mut buf = String::new();
        for (n, error) in errors {
            buf.clear();
            write!(&mut buf, "{}", error).unwrap();
            writeln!(f)?;
            write!(indented(f).ind(n), "{}", Red.make_intense().paint(&buf))?;
        }

        let separated = &mut HeaderWriter {
            inner: &mut *f,
            header: &"\n\n",
            started: false,
        };

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
        {
            if let Some(span_trace) = self
                .span_trace
                .as_ref()
                .or_else(|| get_deepest_spantrace(error))
            {
                write!(
                    &mut separated.ready(),
                    "{}",
                    crate::writers::FormattedSpanTrace(span_trace)
                )?;
            }
        }

        if let Some(backtrace) = self.backtrace.as_ref() {
            let fmted_bt = installed_printer().format_backtrace(&backtrace);

            write!(
                indented(&mut separated.ready()).with_format(Format::Uniform { indentation: "  " }),
                "{}",
                fmted_bt
            )?;
        } else if self
            .sections
            .iter()
            .any(|s| !matches!(s, HelpInfo::Custom(_) | HelpInfo::Error(_)))
        {
            writeln!(f)?;
        }

        for section in self
            .sections
            .iter()
            .filter(|s| !matches!(s, HelpInfo::Custom(_) | HelpInfo::Error(_)))
        {
            write!(f, "\n{}", section)?;
        }

        Ok(())
    }
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

#[cfg(feature = "capture-spantrace")]
pub(crate) fn get_deepest_spantrace<'a>(
    error: &'a (dyn std::error::Error + 'static),
) -> Option<&'a SpanTrace> {
    eyre::Chain::new(error)
        .rev()
        .flat_map(|error| error.span_trace())
        .next()
}
