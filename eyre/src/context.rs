use crate::error::{ContextError, ErrorImpl};
use crate::{Report, StdError, WrapErr};
use core::fmt::{self, Debug, Display, Write};

mod ext {
    use super::*;

    pub trait StdError {
        #[cfg_attr(track_caller, track_caller)]
        fn ext_report<D>(self, msg: D) -> Report
        where
            D: Display + Send + Sync + 'static;
    }

    impl<E> StdError for E
    where
        E: std::error::Error + Send + Sync + 'static,
    {
        fn ext_report<D>(self, msg: D) -> Report
        where
            D: Display + Send + Sync + 'static,
        {
            Report::from_msg(msg, self)
        }
    }

    impl StdError for Report {
        fn ext_report<D>(self, msg: D) -> Report
        where
            D: Display + Send + Sync + 'static,
        {
            self.wrap_err(msg)
        }
    }
}

impl<T, E> WrapErr<T, E> for Result<T, E>
where
    E: ext::StdError + Send + Sync + 'static,
{
    fn wrap_err<D>(self, msg: D) -> Result<T, Report>
    where
        D: Display + Send + Sync + 'static,
    {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(e.ext_report(msg)),
        }
    }

    fn wrap_err_with<D, F>(self, msg: F) -> Result<T, Report>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D,
    {
        match self {
            Ok(t) => Ok(t),
            Err(e) => Err(e.ext_report(msg())),
        }
    }
}

#[cfg(feature = "anyhow")]
impl<T, E> crate::ContextCompat<T> for Result<T, E>
where
    Self: WrapErr<T, E>,
{
    #[track_caller]
    fn context<D>(self, msg: D) -> crate::Result<T, Report>
    where
        D: Display + Send + Sync + 'static,
    {
        self.wrap_err(msg)
    }

    #[track_caller]
    fn with_context<D, F>(self, f: F) -> crate::Result<T, Report>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D,
    {
        self.wrap_err_with(f)
    }
}

#[cfg(feature = "anyhow")]
impl<T> crate::ContextCompat<T> for Option<T> {
    #[track_caller]
    fn context<D>(self, msg: D) -> Result<T, Report>
    where
        D: Display + Send + Sync + 'static,
    {
        match self {
            Some(t) => Ok(t),
            None => Err(Report::from_display(msg)),
        }
    }

    #[track_caller]
    fn with_context<D, F>(self, msg: F) -> Result<T, Report>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D,
    {
        match self {
            Some(t) => Ok(t),
            None => Err(Report::from_display(msg())),
        }
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
    #[cfg(generic_member_access)]
    fn provide<'a>(&'a self, request: &mut std::error::Request<'a>) {
        self.error.provide(request);
    }

    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(&self.error)
    }
}

impl<D> StdError for ContextError<D, Report>
where
    D: Display,
{
    #[cfg(generic_member_access)]
    fn provide<'a>(&'a self, request: &mut std::error::Request<'a>) {
        self.error.provide(request)
    }

    fn source(&self) -> Option<&(dyn StdError + 'static)> {
        Some(ErrorImpl::error(self.error.inner.as_ref()))
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

    pub trait Sealed {}

    impl<T, E> Sealed for Result<T, E> where E: ext::StdError {}
    impl<T> Sealed for Option<T> {}
}
