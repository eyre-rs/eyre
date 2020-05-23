use crate::error::ErrorImpl;
use crate::EyreContext;
use core::fmt;

impl<C> ErrorImpl<(), C>
where
    C: EyreContext,
{
    pub(crate) fn display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.context
            .as_ref()
            .map(|context| context.display(self.error(), f))
            .unwrap_or_else(|| core::fmt::Display::fmt(self.error(), f))
    }

    pub(crate) fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        self.context
            .as_ref()
            .map(|context| context.debug(self.error(), f))
            .unwrap_or_else(|| core::fmt::Debug::fmt(self.error(), f))
    }
}
