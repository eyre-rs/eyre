//! Module for new types that isolate complext formatting
use crate::style;
use std::fmt;

pub(crate) struct LocationSection<'a>(
    pub(crate) Option<&'a std::panic::Location<'a>>,
    pub(crate) crate::config::Theme,
);

impl fmt::Display for LocationSection<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let theme = self.1;
        // If known, print panic location.
        if let Some(loc) = self.0 {
            write!(f, "{}", style(loc.file(), theme.panic_file))?;
            write!(f, ":")?;
            write!(f, "{}", style(loc.line(), theme.panic_line_number))?;
        } else {
            write!(f, "<unknown>")?;
        }

        Ok(())
    }
}
