//! Provides an extension trait for attaching `Section` to error reports.
use crate::{
    eyre::{Report, Result},
    ColorExt, Section,
};
use ansi_term::Color::*;
use indenter::indented;
use std::fmt::Write;
use std::fmt::{self, Display};

impl<T, E> Section<T> for std::result::Result<T, E>
where
    E: Into<Report>,
{
    fn note<D>(self, note: D) -> Result<T>
    where
        D: Display + Send + Sync + 'static,
    {
        self.map_err(|e| {
            let mut e = e.into();

            if let Some(handler) = e.handler_mut().downcast_mut::<crate::Handler>() {
                handler.sections.push(HelpInfo::Note(Box::new(note)));
            }

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

            if let Some(handler) = e.handler_mut().downcast_mut::<crate::Handler>() {
                handler.sections.push(HelpInfo::Note(Box::new(note())));
            }

            e
        })
    }

    fn warning<D>(self, warning: D) -> Result<T>
    where
        D: Display + Send + Sync + 'static,
    {
        self.map_err(|e| {
            let mut e = e.into();

            if let Some(handler) = e.handler_mut().downcast_mut::<crate::Handler>() {
                handler.sections.push(HelpInfo::Warning(Box::new(warning)));
            }

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

            if let Some(handler) = e.handler_mut().downcast_mut::<crate::Handler>() {
                handler
                    .sections
                    .push(HelpInfo::Warning(Box::new(warning())));
            }

            e
        })
    }

    fn suggestion<D>(self, suggestion: D) -> Result<T>
    where
        D: Display + Send + Sync + 'static,
    {
        self.map_err(|e| {
            let mut e = e.into();

            if let Some(handler) = e.handler_mut().downcast_mut::<crate::Handler>() {
                handler
                    .sections
                    .push(HelpInfo::Suggestion(Box::new(suggestion)));
            }

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

            if let Some(handler) = e.handler_mut().downcast_mut::<crate::Handler>() {
                handler
                    .sections
                    .push(HelpInfo::Suggestion(Box::new(suggestion())));
            }

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

            if let Some(handler) = e.handler_mut().downcast_mut::<crate::Handler>() {
                let section = Box::new(section());
                handler.sections.push(HelpInfo::Custom(section));
            }

            e
        })
    }

    fn section<D>(self, section: D) -> Result<T>
    where
        D: Display + Send + Sync + 'static,
    {
        self.map_err(|e| {
            let mut e = e.into();

            if let Some(handler) = e.handler_mut().downcast_mut::<crate::Handler>() {
                let section = Box::new(section);
                handler.sections.push(HelpInfo::Custom(section));
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

            if let Some(handler) = e.handler_mut().downcast_mut::<crate::Handler>() {
                let error = error.into();
                handler.sections.push(HelpInfo::Error(error));
            }

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

            if let Some(handler) = e.handler_mut().downcast_mut::<crate::Handler>() {
                let error = error().into();
                handler.sections.push(HelpInfo::Error(error));
            }

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
                    write!(indented(f).ind(n), "{}", Red.make_intense().paint(&buf))?;
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
