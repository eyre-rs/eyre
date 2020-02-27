use crate::error::ContextError;
use crate::{Report, DefaultContext, ErrReport, StdError, EyreContext};
use core::convert::Infallible;
use core::fmt::{self, Debug, Display, Write};

#[cfg(backtrace)]
use std::backtrace::Backtrace;

mod ext {
    use super::*;

    pub trait StdError {
        fn ext_context<D>(self, context: D) -> ErrReport<DefaultContext>
        where
            D: Display + Send + Sync + 'static;
    }

    #[cfg(feature = "std")]
    impl<E> StdError for E
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        fn ext_context<D>(self, context: D) -> ErrReport<DefaultContext>
        where
            D: Display + Send + Sync + 'static,
        {
            ErrReport::from_context(context, self)
        }
    }

    impl StdError for ErrReport<DefaultContext> {
        fn ext_context<D>(self, context: D) -> ErrReport<DefaultContext>
        where
            D: Display + Send + Sync + 'static,
        {
            self.context(context)
        }
    }
}

impl<T, E> Report<T, E> for Result<T, E>
where
    E: ext::StdError + Send + Sync + 'static,
{
    fn context<D>(self, context: D) -> Result<T, ErrReport<DefaultContext>>
    where
        D: Display + Send + Sync + 'static,
    {
        self.map_err(|error| error.ext_context(context))
    }

    fn with_context<D, F>(self, context: F) -> Result<T, ErrReport<DefaultContext>>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D,
    {
        self.map_err(|error| error.ext_context(context()))
    }
}

/// ```
/// # type T = ();
/// #
/// use eyre::{Report, Result};
///
/// fn maybe_get() -> Option<T> {
///     # const IGNORE: &str = stringify! {
///     ...
///     # };
///     # unimplemented!()
/// }
///
/// fn demo() -> Result<()> {
///     let t = maybe_get().context("there is no T")?;
///     # const IGNORE: &str = stringify! {
///     ...
///     # };
///     # unimplemented!()
/// }
/// ```
impl<T> Report<T, Infallible> for Option<T> {
    fn context<D>(self, context: D) -> Result<T, ErrReport<DefaultContext>>
    where
        D: Display + Send + Sync + 'static,
    {
        self.ok_or_else(|| ErrReport::from_display(context))
    }

    fn with_context<D, F>(self, context: F) -> Result<T, ErrReport<DefaultContext>>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D,
    {
        self.ok_or_else(|| ErrReport::from_display(context()))
    }
}

impl<D, E> Debug for ContextError<D, E>
where
    D: Display,
    E: Debug,
{
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
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

impl<D, C> StdError for ContextError<D, ErrReport<C>>
where
    C: EyreContext,
    D: Display,
{
    #[cfg(backtrace)]
    fn backtrace(&self) -> Option<&Backtrace> {
        Some(self.error.backtrace())
    }

    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(self.error.inner.error())
    }
}

struct Quoted<D>(D);

impl<D> Debug for Quoted<D>
where
    D: Display,
{
    fn fmt(&self, formatter: &mut fmt::Formatter) -> fmt::Result {
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

    pub trait Sealed {}

    impl<T, E> Sealed for Result<T, E> where E: ext::StdError {}
    impl<T> Sealed for Option<T> {}
}
