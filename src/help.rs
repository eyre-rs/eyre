use crate::{Report, Result};
use console::style;
use std::fmt::{self, Display};

/// A helper trait for attaching help text to errors to be displayed after the chain of errors
pub trait Help<T>: private::Sealed {
    /// Add a note to an error, to be displayed after the chain of errors.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use std::{error::Error, fmt::{self, Display}};
    /// # use jane_eyre::Result;
    /// # #[derive(Debug)]
    /// # struct FakeErr;
    /// # impl Display for FakeErr {
    /// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    /// #         write!(f, "FakeErr")
    /// #     }
    /// # }
    /// # impl std::error::Error for FakeErr {}
    /// # fn main() -> Result<()> {
    /// # fn fallible_fn() -> Result<(), FakeErr> {
    /// #       Ok(())
    /// # }
    /// use jane_eyre::Help as _;
    ///
    /// fallible_fn().note("This might have failed due to ...")?;
    /// # Ok(())
    /// # }
    /// ```
    fn note<C>(self, context: C) -> Result<T>
    where
        C: Display + Send + Sync + 'static;

    /// Add a Note to an error, to be displayed after the chain of errors, which is lazily
    /// evaluated only in the case of an error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use std::{error::Error, fmt::{self, Display}};
    /// # use jane_eyre::Result;
    /// # #[derive(Debug)]
    /// # struct FakeErr;
    /// # impl Display for FakeErr {
    /// #     fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
    /// #         write!(f, "FakeErr")
    /// #     }
    /// # }
    /// # impl std::error::Error for FakeErr {}
    /// # fn main() -> Result<()> {
    /// # fn fallible_fn() -> Result<(), FakeErr> {
    /// #       Ok(())
    /// # }
    /// use jane_eyre::Help as _;
    ///
    /// fallible_fn().with_note(|| {
    ///         format!("This might have failed due to ... It has failed {} times", 100)
    ///     })?;
    /// # Ok(())
    /// # }
    /// ```
    fn with_note<C, F>(self, f: F) -> Result<T>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C;

    /// Add a Warning to an error, to be displayed after the chain of errors.
    fn warning<C>(self, context: C) -> Result<T>
    where
        C: Display + Send + Sync + 'static;

    /// Add a Warning to an error, to be displayed after the chain of errors, which is lazily
    /// evaluated only in the case of an error.
    fn with_warning<C, F>(self, f: F) -> Result<T>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C;

    /// Add a Suggestion to an error, to be displayed after the chain of errors.
    fn suggestion<C>(self, context: C) -> Result<T>
    where
        C: Display + Send + Sync + 'static;

    /// Add a Suggestion to an error, to be displayed after the chain of errors, which is lazily
    /// evaluated only in the case of an error.
    fn with_suggestion<C, F>(self, f: F) -> Result<T>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C;
}

impl<T, E> Help<T> for std::result::Result<T, E>
where
    E: Into<Report>,
{
    fn note<C>(self, context: C) -> Result<T>
    where
        C: Display + Send + Sync + 'static,
    {
        self.map_err(|e| {
            let mut e = e.into();
            e.context_mut().help.push(HelpInfo::Note(Box::new(context)));
            e
        })
    }

    fn with_note<C, F>(self, context: F) -> Result<T>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_err(|e| {
            let mut e = e.into();
            e.context_mut()
                .help
                .push(HelpInfo::Note(Box::new(context())));
            e
        })
    }

    fn warning<C>(self, context: C) -> Result<T>
    where
        C: Display + Send + Sync + 'static,
    {
        self.map_err(|e| {
            let mut e = e.into();
            e.context_mut()
                .help
                .push(HelpInfo::Warning(Box::new(context)));
            e
        })
    }

    fn with_warning<C, F>(self, context: F) -> Result<T>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_err(|e| {
            let mut e = e.into();
            e.context_mut()
                .help
                .push(HelpInfo::Warning(Box::new(context())));
            e
        })
    }

    fn suggestion<C>(self, context: C) -> Result<T>
    where
        C: Display + Send + Sync + 'static,
    {
        self.map_err(|e| {
            let mut e = e.into();
            e.context_mut()
                .help
                .push(HelpInfo::Suggestion(Box::new(context)));
            e
        })
    }

    fn with_suggestion<C, F>(self, context: F) -> Result<T>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C,
    {
        self.map_err(|e| {
            let mut e = e.into();
            e.context_mut()
                .help
                .push(HelpInfo::Suggestion(Box::new(context())));
            e
        })
    }
}

pub enum HelpInfo {
    Note(Box<dyn Display + Send + Sync + 'static>),
    Warning(Box<dyn Display + Send + Sync + 'static>),
    Suggestion(Box<dyn Display + Send + Sync + 'static>),
}

impl Display for HelpInfo {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        match self {
            Self::Note(context) => write!(f, "Note: {}", context),
            Self::Warning(context) => write!(f, "Warning: {}", context),
            Self::Suggestion(context) => write!(f, "{}: {}", style("Suggestion").cyan(), context),
        }
    }
}

pub(crate) mod private {
    use crate::Report;
    pub trait Sealed {}

    impl<T, E> Sealed for std::result::Result<T, E> where E: Into<Report> {}
}
