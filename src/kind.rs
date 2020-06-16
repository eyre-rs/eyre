#![allow(missing_debug_implementations, missing_docs)]
// Tagged dispatch mechanism for resolving the behavior of `eyre!($expr)`.
//
// When eyre! is given a single expr argument to turn into eyre::Report, we
// want the resulting Report to pick up the input's implementation of source()
// and backtrace() if it has a std::error::Error impl, otherwise require nothing
// more than Display and Debug.
//
// Expressed in terms of specialization, we want something like:
//
//     trait EyreNew {
//         fn new(self) -> Report;
//     }
//
//     impl<T> EyreNew for T
//     where
//         T: Display + Debug + Send + Sync + 'static,
//     {
//         default fn new(self) -> Report {
//             /* no std error impl */
//         }
//     }
//
//     impl<T> EyreNew for T
//     where
//         T: std::error::Error + Send + Sync + 'static,
//     {
//         fn new(self) -> Report {
//             /* use std error's source() and backtrace() */
//         }
//     }
//
// Since specialization is not stable yet, instead we rely on autoref behavior
// of method resolution to perform tagged dispatch. Here we have two traits
// AdhocKind and TraitKind that both have an eyre_kind() method. AdhocKind is
// implemented whether or not the caller's type has a std error impl, while
// TraitKind is implemented only when a std error impl does exist. The ambiguity
// is resolved by AdhocKind requiring an extra autoref so that it has lower
// precedence.
//
// The eyre! macro will set up the call in this form:
//
//     #[allow(unused_imports)]
//     use $crate::private::{AdhocKind, TraitKind};
//     let error = $msg;
//     (&error).eyre_kind().new(error)

use crate::{EyreHandler, Report};
use core::fmt::{Debug, Display};

#[cfg(feature = "std")]
use crate::StdError;

// #[cfg(backtrace)]
// use std::backtrace::Backtrace;

pub struct Adhoc;

pub trait AdhocKind: Sized {
    #[inline]
    fn eyre_kind(&self) -> Adhoc {
        Adhoc
    }
}

impl<T> AdhocKind for &T where T: ?Sized + Display + Debug + Send + Sync + 'static {}

impl Adhoc {
    pub fn new<M, H: EyreHandler>(self, message: M) -> Report<H>
    where
        M: Display + Debug + Send + Sync + 'static,
    {
        Report::from_adhoc(message)
    }
}

pub struct Trait<H>(std::marker::PhantomData<H>);

pub trait TraitKind<H>: Sized {
    #[inline]
    fn eyre_kind(&self) -> Trait<H> {
        Trait(std::marker::PhantomData)
    }
}

impl<E, H> TraitKind<H> for E
where
    E: Into<Report<H>>,
    H: EyreHandler,
{
}

impl<H> Trait<H> {
    pub fn new<E>(self, error: E) -> Report<H>
    where
        E: Into<Report<H>>,
        H: EyreHandler,
    {
        error.into()
    }
}

#[cfg(feature = "std")]
pub struct Boxed;

#[cfg(feature = "std")]
pub trait BoxedKind: Sized {
    #[inline]
    fn eyre_kind(&self) -> Boxed {
        Boxed
    }
}

#[cfg(feature = "std")]
impl BoxedKind for Box<dyn StdError + Send + Sync> {}

#[cfg(feature = "std")]
impl Boxed {
    pub fn new<H: EyreHandler>(self, error: Box<dyn StdError + Send + Sync>) -> Report<H> {
        Report::from_boxed(error)
    }
}

#[cfg(test)]
mod test {
    use super::*;

    use crate::{eyre, EyreHandler, Report};

    struct NonDefaultHandler;

    impl EyreHandler for NonDefaultHandler {
        #[allow(unused_variables)]
        fn default(error: &(dyn StdError + 'static)) -> Self {
            Self
        }

        fn debug(
            &self,
            _error: &(dyn StdError + 'static),
            _f: &mut core::fmt::Formatter<'_>,
        ) -> core::fmt::Result {
            Ok(())
        }
    }

    // This has to compile without changes
    fn _throw_error<E>(e: E) -> Report<NonDefaultHandler>
    where
        E: Into<Report<NonDefaultHandler>>,
    {
        eyre!(e).wrap_err("blaah")
    }
}
