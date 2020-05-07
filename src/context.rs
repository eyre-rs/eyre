use crate::error::ContextError;
use crate::{ContextCompat, EyreContext, Report, StdError, WrapErr};
use core::fmt::{self, Debug, Display, Write};

#[cfg(backtrace)]
use std::backtrace::Backtrace;

mod ext {
    use super::*;

    pub trait StdError<C>
    where
        C: EyreContext,
    {
        fn ext_report<D>(self, msg: D) -> Report<C>
        where
            D: Display + Send + Sync + 'static;
    }

    #[cfg(feature = "std")]
    impl<E, C> StdError<C> for E
    where
        C: EyreContext,
        E: std::error::Error + Send + Sync + 'static,
    {
        fn ext_report<D>(self, msg: D) -> Report<C>
        where
            D: Display + Send + Sync + 'static,
        {
            Report::from_msg(msg, self)
        }
    }

    impl<C> StdError<C> for Report<C>
    where
        C: EyreContext,
    {
        fn ext_report<D>(self, msg: D) -> Report<C>
        where
            D: Display + Send + Sync + 'static,
        {
            self.wrap_err(msg)
        }
    }
}

impl<T, E, C> WrapErr<T, E, C> for Result<T, E>
where
    C: EyreContext,
    E: ext::StdError<C> + Send + Sync + 'static,
{
    fn wrap_err<D>(self, msg: D) -> Result<T, Report<C>>
    where
        D: Display + Send + Sync + 'static,
    {
        self.map_err(|error| error.ext_report(msg))
    }

    fn wrap_err_with<D, F>(self, msg: F) -> Result<T, Report<C>>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D,
    {
        self.map_err(|error| error.ext_report(msg()))
    }

    fn context<D>(self, msg: D) -> Result<T, Report<C>>
    where
        D: Display + Send + Sync + 'static,
    {
        self.wrap_err(msg)
    }

    fn with_context<D, F>(self, msg: F) -> Result<T, Report<C>>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D,
    {
        self.wrap_err_with(msg)
    }
}

impl<T, C> ContextCompat<T, C> for Option<T>
where
    C: EyreContext,
{
    fn wrap_err<D>(self, msg: D) -> Result<T, Report<C>>
    where
        D: Display + Send + Sync + 'static,
    {
        self.context(msg)
    }

    fn wrap_err_with<D, F>(self, msg: F) -> Result<T, Report<C>>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D,
    {
        self.with_context(msg)
    }

    fn context<D>(self, msg: D) -> Result<T, Report<C>>
    where
        D: Display + Send + Sync + 'static,
    {
        self.ok_or_else(|| Report::from_display(msg))
    }

    fn with_context<D, F>(self, msg: F) -> Result<T, Report<C>>
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

impl<D, C> StdError for ContextError<D, Report<C>>
where
    C: EyreContext,
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

    pub trait Sealed<C: EyreContext> {}

    impl<T, E, C: EyreContext> Sealed<C> for Result<T, E> where E: ext::StdError<C> {}
    impl<T, C: EyreContext> Sealed<C> for Option<T> {}
}
