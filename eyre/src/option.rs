use crate::OptionExt;
use core::fmt::{Debug, Display};

impl<T> OptionExt<T> for Option<T> {
    #[track_caller]
    fn ok_or_eyre<M>(self, message: M) -> crate::Result<T>
    where
        M: Debug + Display + Send + Sync + 'static,
    {
        match self {
            Some(ok) => Ok(ok),
            None => Err(crate::Report::msg(message)),
        }
    }

    #[track_caller]
    fn die<D>(self, msg: D) -> T
    where
        D: Debug + Display + Send + Sync + 'static
    {
        match self {
            Some(ok) => ok,
            None => std::panic::panic_any(crate::Report::msg(msg))
        }
    }

    #[track_caller]
    fn die_with<D, F>(self, msg: F) -> T
    where
        D: Debug + Display + Send + Sync + 'static,
        F: FnOnce() -> D
    {
        match self {
            Some(ok) => ok,
            None => std::panic::panic_any(crate::Report::msg(msg()))
        }
    }
}
