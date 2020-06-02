use crate::error::ContextError;
use crate::{ContextCompat, EyreHandler, Report, StdError, WrapErr};
use core::fmt::{self, Debug, Display, Write};

#[cfg(backtrace)]
use std::backtrace::Backtrace;

mod ext {
    use super::*;

    pub trait StdError<H>
    where
        H: EyreHandler,
    {
        fn ext_report<D>(self, msg: D) -> Report<H>
        where
            D: Display + Send + Sync + 'static;
    }

    #[cfg(feature = "std")]
    impl<E, H> StdError<H> for E
    where
        H: EyreHandler,
        E: std::error::Error + Send + Sync + 'static,
    {
        fn ext_report<D>(self, msg: D) -> Report<H>
        where
            D: Display + Send + Sync + 'static,
        {
            Report::from_msg(msg, self)
        }
    }

    impl<H> StdError<H> for Report<H>
    where
        H: EyreHandler,
    {
        fn ext_report<D>(self, msg: D) -> Report<H>
        where
            D: Display + Send + Sync + 'static,
        {
            self.wrap_err(msg)
        }
    }
}

impl<T, E, H> WrapErr<T, E, H> for Result<T, E>
where
    H: EyreHandler,
    E: ext::StdError<H> + Send + Sync + 'static,
{
    fn wrap_err<D>(self, msg: D) -> Result<T, Report<H>>
    where
        D: Display + Send + Sync + 'static,
    {
        self.map_err(|error| error.ext_report(msg))
    }

    fn wrap_err_with<D, F>(self, msg: F) -> Result<T, Report<H>>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D,
    {
        self.map_err(|error| error.ext_report(msg()))
    }

    fn context<D>(self, msg: D) -> Result<T, Report<H>>
    where
        D: Display + Send + Sync + 'static,
    {
        self.wrap_err(msg)
    }

    fn with_context<D, F>(self, msg: F) -> Result<T, Report<H>>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D,
    {
        self.wrap_err_with(msg)
    }
}

impl<T, H> ContextCompat<T, H> for Option<T>
where
    H: EyreHandler,
{
    fn wrap_err<D>(self, msg: D) -> Result<T, Report<H>>
    where
        D: Display + Send + Sync + 'static,
    {
        self.context(msg)
    }

    fn wrap_err_with<D, F>(self, msg: F) -> Result<T, Report<H>>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D,
    {
        self.with_context(msg)
    }

    fn context<D>(self, msg: D) -> Result<T, Report<H>>
    where
        D: Display + Send + Sync + 'static,
    {
        self.ok_or_else(|| Report::from_display(msg))
    }

    fn with_context<D, F>(self, msg: F) -> Result<T, Report<H>>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D,
    {
        self.ok_or_else(|| Report::from_display(msg()))
    }
}

impl<D, E> Debug for ContextError<D, E>
where
    D: Display,
    E: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("Error")
            .field("msg", &Quoted(&self.msg))
            .field("source", &self.error)
            .finish()
    }
}

impl<D, E> Display for ContextError<D, E>
where
    D: Display,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        Display::fmt(&self.msg, f)
    }
}

impl<D, E> StdError for ContextError<D, E>
where
    D: Display,
    E: StdError + 'static,
{
    #[cfg(backtrace)]
    fn backtrace(&self) -> Option<&Backtrace> {
        self.error.backtrace()
    }

    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.error)
    }
}

impl<D, H> StdError for ContextError<D, Report<H>>
where
    H: EyreHandler,
    D: Display,
{
    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(self.error.inner.error())
    }
}

struct Quoted<D>(D);

impl<D> Debug for Quoted<D>
where
    D: Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter<'_>) -> fmt::Result {
        formatter.write_char('"')?;
        Quoted(&mut *formatter).write_fmt(format_args!("{}", self.0))?;
        formatter.write_char('"')?;
        Ok(())
    }
}

impl Write for Quoted<&mut fmt::Formatter<'_>> {
    fn write_str(&mut self, s: &str) -> fmt::Result {
        Display::fmt(&s.escape_debug(), self.0)
    }
}

pub(crate) mod private {
    use super::*;

    pub trait Sealed<H: EyreHandler> {}

    impl<T, E, H: EyreHandler> Sealed<H> for Result<T, E> where E: ext::StdError<H> {}
    impl<T, H: EyreHandler> Sealed<H> for Option<T> {}
}
