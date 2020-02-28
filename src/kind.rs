// Tagged dispatch mechanism for resolving the behavior of `eyre!($expr)`.
//
// When eyre! is given a single expr argument to turn into eyre::ErrReport, we
// want the resulting ErrReport to pick up the input's implementation of source()
// and backtrace() if it has a std::error::Error impl, otherwise require nothing
// more than Display and Debug.
//
// Expressed in terms of specialization, we want something like:
//
//     trait EyreNew {
//         fn new(self) -> ErrReport;
//     }
//
//     impl<T> EyreNew for T
//     where
//         T: Display + Debug + Send + Sync + 'static,
//     {
//         default fn new(self) -> ErrReport {
//             /* no std error impl */
//         }
//     }
//
//     impl<T> EyreNew for T
//     where
//         T: std::error::Error + Send + Sync + 'static,
//     {
//         fn new(self) -> ErrReport {
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

use crate::{ErrReport, EyreContext};
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
    pub fn new<M, C: EyreContext>(self, message: M) -> ErrReport<C>
    where
        M: Display + Debug + Send + Sync + 'static,
    {
        ErrReport::from_adhoc(message)
    }
}

pub struct Trait;

pub trait TraitKind: Sized {
    #[inline]
    fn eyre_kind(&self) -> Trait {
        Trait
    }
}

impl<E> TraitKind for E where E: Into<ErrReport> {}

impl Trait {
    pub fn new<E>(self, error: E) -> ErrReport
    where
        E: Into<ErrReport>,
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
    pub fn new<C: EyreContext>(self, error: Box<dyn StdError + Send + Sync>) -> ErrReport<C> {
        ErrReport::from_boxed(error)
    }
}
