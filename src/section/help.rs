//! Provides an extension trait for attaching `Section`s to error reports.
use crate::{section, Report, Result, Section};
use ansi_term::Color::*;
use std::fmt::{self, Display};

/// A helper trait for attaching help text to errors to be displayed after the chain of errors
///
/// `color_eyre` provides two types of help text that can be attached to error reports: custom
/// sections and pre-configured sections. Custom sections are added via the `section` and
/// `with_section` methods, and give maximum control over formatting. For more details check out
/// the docs for [`Section`].
///
/// The pre-configured sections are provided via `suggestion`, `warning`, and `note`. These
/// sections are displayed after all other sections with no extra newlines between subsequent Help
/// sections. They consist only of a header portion and are prepended with a colored string
/// indicating the kind of section, e.g. `Note: This might have failed due to ..."
///
/// [`Section`]: struct.Section.html
pub trait Help<T>: private::Sealed {
    /// Add a section to an error report, to be displayed after the chain of errors.
    ///
    /// Sections are displayed in the order they are added to the error report. They are displayed
    /// immediately after the `Error:` section and before the `SpanTrace` and `Backtrace` sections.
    /// They consist of a header and an optional body. The body of the section is indented by
    /// default.
    ///
    /// # Examples
    ///
    /// ```rust,should_panic
    /// use color_eyre::{Report, Help};
    /// use eyre::eyre;
    ///
    /// Err(eyre!("command failed"))
    ///     .section("Please report bugs to https://real.url/bugs")?;
    /// # Ok::<_, Report>(())
    /// ```
    fn section<C>(self, section: C) -> Result<T>
    where
        C: Into<Section>;

    /// Add a Section to an error report, to be displayed after the chain of errors. The closure to
    /// create the Section is lazily evaluated only in the case of an error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use color_eyre::{Report, Help, SectionExt};
    /// use eyre::eyre;
    ///
    /// let output = std::process::Command::new("ls")
    ///     .output()?;
    ///
    /// let output = if !output.status.success() {
    ///     let stderr = String::from_utf8_lossy(&output.stderr);
    ///     Err(eyre!("cmd exited with non-zero status code"))
    ///         .with_section(move || {
    ///             "Stderr:"
    ///                 .skip_if(|| stderr.is_empty())
    ///                 .body(stderr.trim().to_string())
    ///         })?
    /// } else {
    ///     String::from_utf8_lossy(&output.stdout)
    /// };
    ///
    /// println!("{}", output);
    /// # Ok::<_, Report>(())
    /// ```
    fn with_section<C, F>(self, section: F) -> Result<T>
    where
        C: Into<Section>,
        F: FnOnce() -> C;

    /// Add an error section to an error report, to be displayed after the primary error message
    /// section.
    ///
    /// # Examples
    ///
    /// ```rust,should_panic
    /// use color_eyre::{Report, Help};
    /// use eyre::eyre;
    /// use thiserror::Error;
    ///
    /// #[derive(Debug, Error)]
    /// #[error("{0}")]
    /// struct StrError(&'static str);
    ///
    /// Err(eyre!("command failed"))
    ///     .error(StrError("got one error"))
    ///     .error(StrError("got a second error"))?;
    /// # Ok::<_, Report>(())
    /// ```
    fn error<E>(self, error: E) -> Result<T>
    where
        E: std::error::Error + Send + Sync + 'static;

    /// Add an error section to an error report, to be displayed after the primary error message
    /// section. The closure to create the Section is lazily evaluated only in the case of an error.
    ///
    /// # Examples
    ///
    /// ```rust,should_panic
    /// use color_eyre::{Report, Help};
    /// use eyre::eyre;
    /// use thiserror::Error;
    ///
    /// #[derive(Debug, Error)]
    /// #[error("{0}")]
    /// struct StringError(String);
    ///
    /// Err(eyre!("command failed"))
    ///     .with_error(|| StringError("got one error".into()))
    ///     .with_error(|| StringError("got a second error".into()))?;
    /// # Ok::<_, Report>(())
    /// ```
    fn with_error<E, F>(self, error: F) -> Result<T>
    where
        F: FnOnce() -> E,
        E: std::error::Error + Send + Sync + 'static;

    /// Add a Note to an error report, to be displayed after the chain of errors.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use std::{error::Error, fmt::{self, Display}};
    /// # use color_eyre::Result;
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
    /// use color_eyre::Help as _;
    ///
    /// fallible_fn().note("This might have failed due to ...")?;
    /// # Ok(())
    /// # }
    /// ```
    fn note<C>(self, context: C) -> Result<T>
    where
        C: Display + Send + Sync + 'static;

    /// Add a Note to an error report, to be displayed after the chain of errors. The closure to
    /// create the Note is lazily evaluated only in the case of an error.
    ///
    /// # Examples
    ///
    /// ```rust
    /// # use std::{error::Error, fmt::{self, Display}};
    /// # use color_eyre::Result;
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
    /// use color_eyre::Help as _;
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

    /// Add a Warning to an error report, to be displayed after the chain of errors.
    fn warning<C>(self, context: C) -> Result<T>
    where
        C: Display + Send + Sync + 'static;

    /// Add a Warning to an error report, to be displayed after the chain of errors. The closure to
    /// create the Warning is lazily evaluated only in the case of an error.
    fn with_warning<C, F>(self, f: F) -> Result<T>
    where
        C: Display + Send + Sync + 'static,
        F: FnOnce() -> C;

    /// Add a Suggestion to an error report, to be displayed after the chain of errors.
    fn suggestion<C>(self, context: C) -> Result<T>
    where
        C: Display + Send + Sync + 'static;

    /// Add a Suggestion to an error report, to be displayed after the chain of errors. The closure
    /// to create the Suggestion is lazily evaluated only in the case of an error.
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
            e.context_mut().sections.push(
                Section::from(HelpInfo::Note(Box::new(context)))
                    .order(section::Order::AfterBackTrace),
            );
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
            e.context_mut().sections.push(
                Section::from(HelpInfo::Note(Box::new(context())))
                    .order(section::Order::AfterBackTrace),
            );
            e
        })
    }

    fn warning<C>(self, context: C) -> Result<T>
    where
        C: Display + Send + Sync + 'static,
    {
        self.map_err(|e| {
            let mut e = e.into();
            e.context_mut().sections.push(
                Section::from(HelpInfo::Warning(Box::new(context)))
                    .order(section::Order::AfterBackTrace),
            );
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
            e.context_mut().sections.push(
                Section::from(HelpInfo::Warning(Box::new(context())))
                    .order(section::Order::AfterBackTrace),
            );
            e
        })
    }

    fn suggestion<C>(self, context: C) -> Result<T>
    where
        C: Display + Send + Sync + 'static,
    {
        self.map_err(|e| {
            let mut e = e.into();
            e.context_mut().sections.push(
                Section::from(HelpInfo::Suggestion(Box::new(context)))
                    .order(section::Order::AfterBackTrace),
            );
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
            e.context_mut().sections.push(
                Section::from(HelpInfo::Suggestion(Box::new(context())))
                    .order(section::Order::AfterBackTrace),
            );
            e
        })
    }

    fn with_section<C, F>(self, section: F) -> Result<T>
    where
        C: Into<Section>,
        F: FnOnce() -> C,
    {
        self.map_err(|e| {
            let mut e = e.into();
            let section = section().into();

            if !matches!(section.order, section::Order::SkipEntirely) {
                e.context_mut().sections.push(section);
            }

            e
        })
    }

    fn section<C>(self, section: C) -> Result<T>
    where
        C: Into<Section>,
    {
        self.map_err(|e| {
            let mut e = e.into();
            let section = section.into();

            if !matches!(section.order, section::Order::SkipEntirely) {
                e.context_mut().sections.push(section);
            }

            e
        })
    }

    fn error<E2>(self, error: E2) -> Result<T>
    where
        E2: std::error::Error + Send + Sync + 'static,
    {
        self.map_err(|e| {
            let mut e = e.into();
            let section = Section {
                inner: section::SectionKind::Error(Box::new(error)),
                order: section::Order::AfterErrMsgs,
            };

            e.context_mut().sections.push(section);
            e
        })
    }

    fn with_error<E2, F>(self, error: F) -> Result<T>
    where
        F: FnOnce() -> E2,
        E2: std::error::Error + Send + Sync + 'static,
    {
        self.map_err(|e| {
            let mut e = e.into();
            let section = Section {
                inner: section::SectionKind::Error(Box::new(error())),
                order: section::Order::AfterErrMsgs,
            };

            e.context_mut().sections.push(section);
            e
        })
    }
}

pub(crate) enum HelpInfo {
    Note(Box<dyn Display + Send + Sync + 'static>),
    Warning(Box<dyn Display + Send + Sync + 'static>),
    Suggestion(Box<dyn Display + Send + Sync + 'static>),
}

impl Display for HelpInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Note(context) => write!(f, "{}: {}", Cyan.paint("Note"), context),
            Self::Warning(context) => write!(f, "{}: {}", Yellow.paint("Warning"), context),
            Self::Suggestion(context) => write!(f, "{}: {}", Cyan.paint("Suggestion"), context),
        }
    }
}

impl fmt::Debug for HelpInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Note(context) => f
                .debug_tuple("Note")
                .field(&format_args!("{}", context))
                .finish(),
            Self::Warning(context) => f
                .debug_tuple("Warning")
                .field(&format_args!("{}", context))
                .finish(),
            Self::Suggestion(context) => f
                .debug_tuple("Suggestion")
                .field(&format_args!("{}", context))
                .finish(),
        }
    }
}

pub(crate) mod private {
    use crate::Report;
    pub trait Sealed {}

    impl<T, E> Sealed for std::result::Result<T, E> where E: Into<Report> {}
}
