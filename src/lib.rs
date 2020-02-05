#![feature(backtrace)]
use eyre_impl::{ErrorReporter, Indented};
use std::backtrace::{Backtrace, BacktraceStatus};
use std::fmt::{self, Write as _};

#[derive(Debug)]
pub struct BoxError(Box<dyn std::error::Error + Send + Sync + 'static>);

impl std::error::Error for BoxError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        self.0.source()
    }
}

impl fmt::Display for BoxError {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        fmt::Display::fmt(&self.0, f)
    }
}

pub struct Context {
    backtrace: Backtrace,
}

impl Default for Context {
    fn default() -> Self {
        Self {
            backtrace: Backtrace::capture(),
        }
    }
}

pub struct ErrReport(ErrorReporter<BoxError, Context>);

impl<E> From<E> for ErrReport
where
    E: std::error::Error + Send + Sync + 'static,
{
    fn from(err: E) -> Self {
        ErrReport(ErrorReporter::from(BoxError(Box::new(err))))
    }
}

impl fmt::Debug for ErrReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        let error = &self.0.error;

        if f.alternate() {
            return fmt::Debug::fmt(error, f);
        }

        let errors = self.0.chain().rev().enumerate();

        writeln!(f)?;

        for (n, error) in errors {
            write!(Indented::numbered(f, n), "{}", error)?;
            writeln!(f)?;
        }

        let backtrace = &self.0.context.backtrace;
        if let BacktraceStatus::Captured = backtrace.status() {
            write!(f, "\n\n{}", backtrace)?;
        }

        Ok(())
    }
}

impl fmt::Display for ErrReport {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "{}", self.0.error)?;

        if f.alternate() {
            for cause in self.0.chain().skip(1) {
                write!(f, ": {}", cause)?;
            }
        }

        Ok(())
    }
}
