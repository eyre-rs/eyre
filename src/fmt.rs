use crate::error::ErrorImpl;
use crate::EyreContext;
use core::fmt;

impl<C> ErrorImpl<(), C>
where
    C: EyreContext,
{
    pub(crate) fn display(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.context.as_ref().unwrap().display(self.error(), f)
    }

    pub(crate) fn debug(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.context.as_ref().unwrap().debug(self.error(), f)
    }
}
