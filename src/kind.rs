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

use crate::Report;
use core::fmt::{Debug, Display};

#[cfg(feature = "std")]
use crate::StdError;

pub struct Adhoc;

pub trait AdhocKind: Sized {
    #[inline]
    fn eyre_kind(&self) -> Adhoc {
        Adhoc
    }
}

impl<T> AdhocKind for &T where T: ?Sized + Display + Debug + Send + Sync + 'static {}

impl Adhoc {
    pub fn new<M>(self, message: M) -> Report
    where
        M: Display + Debug + Send + Sync + 'static,
    {
        Report::from_adhoc(message)
    }
}

pub struct Trait;

pub trait TraitKind: Sized {
    #[inline]
    fn eyre_kind(&self) -> Trait {
        Trait
    }
}

impl<E> TraitKind for E where E: Into<Report> {}

impl Trait {
    pub fn new<E>(self, error: E) -> Report
    where
        E: Into<Report>,
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
    pub fn new(self, error: Box<dyn StdError + Send + Sync>) -> Report {
        Report::from_boxed(error)
    }
}

#[cfg(test)]
mod test {
    use super::*;
    use crate::eyre;
    use crate::EyreHandler;
    use std::num::ParseIntError;

    struct NonDefaultHandler;

    impl EyreHandler for NonDefaultHandler {
        fn debug(
            &self,
            _error: &(dyn StdError + 'static),
            _f: &mut core::fmt::Formatter<'_>,
        ) -> core::fmt::Result {
            Ok(())
        }
    }

    fn _parse(s: &str) -> Result<i32, ParseIntError> {
        s.parse::<i32>()
    }

    fn _throw_error() -> Result<(), Report> {
        match _parse("abc") {
            Ok(_) => Ok(()),
            Err(e) => Err(eyre!(e).wrap_err("try parsing an actual number")),
        }
    }
}
