use crate::error::ErrorImpl;
use core::fmt;

impl ErrorImpl<()> {
    pub(crate) fn display(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.handlers.is_empty() {
            core::fmt::Display::fmt(self.error(), f)?;
        } else {
            for handler in &self.handlers {
                handler.display(self.error(), f)?;
            }
        }

        Ok(())
    }

    pub(crate) fn debug(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        if self.handlers.is_empty() {
            core::fmt::Debug::fmt(self.error(), f)?;
        } else {
            for handler in &self.handlers {
                handler.debug(self.error(), f)?;
            }
        }

        Ok(())
    }
}
