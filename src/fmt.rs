use crate::error::ErrorImpl;
use crate::EyreContext;
use core::fmt;

impl<C> ErrorImpl<(), C>
where
    C: EyreContext,
{
    pub(crate) fn display(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.context.display(self.error(), f)
    }

    pub(crate) fn debug(&self, f: &mut fmt::Formatter) -> fmt::Result {
        self.context.debug(self.error(), f)
    }
}
