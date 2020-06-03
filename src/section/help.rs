//! Provides an extension trait for attaching `Section` to error reports.
use crate::ColorExt;
use crate::{Report, Result};
use ansi_term::Color::*;
use indenter::indented;
use std::fmt::Write;
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
    fn section<D>(self, section: D) -> Result<T>
    where
        D: Display + Send + Sync + 'static;

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
    ///         .with_section(move || stderr.trim().to_string().header("Stderr:"))?
    /// } else {
    ///     String::from_utf8_lossy(&output.stdout)
    /// };
    ///
    /// println!("{}", output);
    /// # Ok::<_, Report>(())
    /// ```
    fn with_section<D, F>(self, section: F) -> Result<T>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D;

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
    fn note<D>(self, note: D) -> Result<T>
    where
        D: Display + Send + Sync + 'static;

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
    fn with_note<D, F>(self, f: F) -> Result<T>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D;

    /// Add a Warning to an error report, to be displayed after the chain of errors.
    fn warning<D>(self, warning: D) -> Result<T>
    where
        D: Display + Send + Sync + 'static;

    /// Add a Warning to an error report, to be displayed after the chain of errors. The closure to
    /// create the Warning is lazily evaluated only in the case of an error.
    fn with_warning<D, F>(self, f: F) -> Result<T>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D;

    /// Add a Suggestion to an error report, to be displayed after the chain of errors.
    fn suggestion<D>(self, suggestion: D) -> Result<T>
    where
        D: Display + Send + Sync + 'static;

    /// Add a Suggestion to an error report, to be displayed after the chain of errors. The closure
    /// to create the Suggestion is lazily evaluated only in the case of an error.
    fn with_suggestion<D, F>(self, f: F) -> Result<T>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D;
}

impl<T, E> Help<T> for std::result::Result<T, E>
where
    E: Into<Report>,
{
    fn note<D>(self, note: D) -> Result<T>
    where
        D: Display + Send + Sync + 'static,
    {
        self.map_err(|e| {
            let mut e = e.into();
            e.handler_mut()
                .sections
                .push(HelpInfo::Note(Box::new(note)));
            e
        })
    }

    fn with_note<D, F>(self, note: F) -> Result<T>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D,
    {
        self.map_err(|e| {
            let mut e = e.into();
            e.handler_mut()
                .sections
                .push(HelpInfo::Note(Box::new(note())));
            e
        })
    }

    fn warning<D>(self, warning: D) -> Result<T>
    where
        D: Display + Send + Sync + 'static,
    {
        self.map_err(|e| {
            let mut e = e.into();
            e.handler_mut()
                .sections
                .push(HelpInfo::Warning(Box::new(warning)));
            e
        })
    }

    fn with_warning<D, F>(self, warning: F) -> Result<T>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D,
    {
        self.map_err(|e| {
            let mut e = e.into();
            e.handler_mut()
                .sections
                .push(HelpInfo::Warning(Box::new(warning())));
            e
        })
    }

    fn suggestion<D>(self, suggestion: D) -> Result<T>
    where
        D: Display + Send + Sync + 'static,
    {
        self.map_err(|e| {
            let mut e = e.into();
            e.handler_mut()
                .sections
                .push(HelpInfo::Suggestion(Box::new(suggestion)));
            e
        })
    }

    fn with_suggestion<D, F>(self, suggestion: F) -> Result<T>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D,
    {
        self.map_err(|e| {
            let mut e = e.into();
            e.handler_mut()
                .sections
                .push(HelpInfo::Suggestion(Box::new(suggestion())));
            e
        })
    }

    fn with_section<D, F>(self, section: F) -> Result<T>
    where
        D: Display + Send + Sync + 'static,
        F: FnOnce() -> D,
    {
        self.map_err(|e| {
            let mut e = e.into();
            let section = Box::new(section());
            e.handler_mut().sections.push(HelpInfo::Custom(section));
            e
        })
    }

    fn section<D>(self, section: D) -> Result<T>
    where
        D: Display + Send + Sync + 'static,
    {
        self.map_err(|e| {
            let mut e = e.into();
            let section = Box::new(section);
            e.handler_mut().sections.push(HelpInfo::Custom(section));
            e
        })
    }

    fn error<E2>(self, error: E2) -> Result<T>
    where
        E2: std::error::Error + Send + Sync + 'static,
    {
        self.map_err(|e| {
            let mut e = e.into();
            let error = error.into();

            e.handler_mut().sections.push(HelpInfo::Error(error));
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
            let error = error().into();

            e.handler_mut().sections.push(HelpInfo::Error(error));
            e
        })
    }
}

pub(crate) enum HelpInfo {
    Error(Box<dyn std::error::Error + Send + Sync + 'static>),
    Custom(Box<dyn Display + Send + Sync + 'static>),
    Note(Box<dyn Display + Send + Sync + 'static>),
    Warning(Box<dyn Display + Send + Sync + 'static>),
    Suggestion(Box<dyn Display + Send + Sync + 'static>),
}

impl Display for HelpInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HelpInfo::Note(note) => write!(f, "{}: {}", Cyan.make_intense().paint("Note"), note),
            HelpInfo::Warning(warning) => {
                write!(f, "{}: {}", Yellow.make_intense().paint("Warning"), warning)
            }
            HelpInfo::Suggestion(suggestion) => write!(
                f,
                "{}: {}",
                Cyan.make_intense().paint("Suggestion"),
                suggestion
            ),
            HelpInfo::Custom(section) => write!(f, "{}", section),
            HelpInfo::Error(error) => {
                // a lot here
                let errors = std::iter::successors(
                    Some(error.as_ref() as &(dyn std::error::Error + 'static)),
                    |e| e.source(),
                );

                write!(f, "Error:")?;
                let mut buf = String::new();
                for (n, error) in errors.enumerate() {
                    writeln!(f)?;
                    buf.clear();
                    write!(&mut buf, "{}", error).unwrap();
                    write!(indented(f).ind(n), "{}", Red.paint(&buf))?;
                }

                Ok(())
            }
        }
    }
}

impl fmt::Debug for HelpInfo {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            HelpInfo::Note(note) => f
                .debug_tuple("Note")
                .field(&format_args!("{}", note))
                .finish(),
            HelpInfo::Warning(warning) => f
                .debug_tuple("Warning")
                .field(&format_args!("{}", warning))
                .finish(),
            HelpInfo::Suggestion(suggestion) => f
                .debug_tuple("Suggestion")
                .field(&format_args!("{}", suggestion))
                .finish(),
            HelpInfo::Custom(custom) => f
                .debug_tuple("CustomSection")
                .field(&format_args!("{}", custom))
                .finish(),
            HelpInfo::Error(error) => f.debug_tuple("Error").field(error).finish(),
        }
    }
}

pub(crate) mod private {
    use crate::Report;
    pub trait Sealed {}

    impl<T, E> Sealed for std::result::Result<T, E> where E: Into<Report> {}
}
