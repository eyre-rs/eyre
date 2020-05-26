//! Helpers for adding custom sections to error reports
use ansi_term::Color::*;
use indenter::indented;
use std::fmt::{self, Display, Write};

pub mod help;

#[non_exhaustive]
#[derive(Debug)]
pub(crate) enum Order {
    AfterErrMsgs,
    BeforeSpanTrace,
    AfterBackTrace,
    SkipEntirely,
}

/// A custom section for an error report.
///
/// # Details
///
/// Sections consist of two parts, a header: and an optional body. The header can contain any
/// number of lines and has no indentation applied to it by default. The body can contain any
/// number of lines and is always written after the header with indentation inserted before
/// every line.
///
/// # Construction
///
/// Sections are meant to be constructed via `Into<Section>`, which is implemented for all types
/// that implement `Display`. The constructed `Section` then takes ownership of the `Display` type
/// and boxes it internally for use later when printing the report.
///
/// # Examples
///
/// ```rust
/// use color_eyre::{SectionExt, Help, Report};
/// use eyre::eyre;
/// use std::process::Command;
/// use tracing::instrument;
///
/// trait Output {
///     fn output2(&mut self) -> Result<String, Report>;
/// }
///
/// impl Output for Command {
///     #[instrument]
///     fn output2(&mut self) -> Result<String, Report> {
///         let output = self.output()?;
///
///         let stdout = String::from_utf8_lossy(&output.stdout);
///
///         if !output.status.success() {
///             let stderr = String::from_utf8_lossy(&output.stderr);
///             Err(eyre!("cmd exited with non-zero status code"))
///                 .with_section(move || {
///                     "Stdout:"
///                         .skip_if(|| stdout.is_empty())
///                         .body(stdout.trim().to_string())
///                 })
///                 .with_section(move || {
///                     "Stderr:"
///                         .skip_if(|| stderr.is_empty())
///                         .body(stderr.trim().to_string())
///                 })
///         } else {
///             Ok(stdout.into())
///         }
///     }
/// }
/// ```
pub struct Section {
    pub(crate) inner: SectionKind,
    pub(crate) order: Order,
}

pub(crate) enum SectionKind {
    Header(Box<dyn Display + Send + Sync + 'static>),
    WithBody(
        Box<dyn Display + Send + Sync + 'static>,
        Box<dyn Display + Send + Sync + 'static>,
    ),
    Error(Box<dyn std::error::Error + Send + Sync + 'static>),
}

/// Extension trait for customizing the content of a `Section`
pub trait SectionExt {
    /// Add a body to a `Section`
    ///
    /// Bodies are always indented to the same level as error messages and spans.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use color_eyre::{Help, SectionExt, Report};
    /// use eyre::eyre;
    ///
    /// let all_in_header = "header\n   body\n   body";
    /// let report = Err::<(), Report>(eyre!("an error occurred"))
    ///     .section(all_in_header)
    ///     .unwrap_err();
    ///
    /// let just_header = "header";
    /// let just_body = "body\nbody";
    /// let report2 = Err::<(), Report>(eyre!("an error occurred"))
    ///     .section(just_header.body(just_body))
    ///     .unwrap_err();
    ///
    /// assert_eq!(format!("{:?}", report), format!("{:?}", report2))
    /// ```
    fn body<C>(self, body: C) -> Section
    where
        C: Display + Send + Sync + 'static;

    /// Skip a section based on some condition. For example, skip a section if the body is empty.
    ///
    /// The skipped section is not stored in the report and is instead immediately dropped.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use eyre::eyre;
    /// use color_eyre::{SectionExt, Report, Help};
    ///
    /// fn add_body(report: Report, body: String) -> Result<(), Report> {
    ///     Err(report)
    ///         .with_section(|| "ExtraInfo:".skip_if(|| body.is_empty()).body(body))
    /// }
    ///
    /// let report = eyre!("an error occurred");
    /// let before = format!("{:?}", report);
    /// let body = String::new();
    /// let report = add_body(report, body).unwrap_err();
    /// let after = format!("{:?}", report);
    /// assert_eq!(before, after);
    ///
    /// let report = eyre!("an error occurred");
    /// let before = format!("{:?}", report);
    /// let body = String::from("Some actual text here");
    /// let report = add_body(report, body).unwrap_err();
    /// let after = format!("{:?}", report);
    /// assert_ne!(before, after);
    /// ```
    fn skip_if<F>(self, condition: F) -> Section
    where
        F: FnOnce() -> bool;
}

impl Section {
    pub(crate) fn order(mut self, order: Order) -> Self {
        self.order = order;
        self
    }
}

impl<T> SectionExt for T
where
    Section: From<T>,
{
    fn body<C>(self, body: C) -> Section
    where
        C: Display + Send + Sync + 'static,
    {
        let section = Section::from(self);

        let header = match section.inner {
            SectionKind::Header(header) => header,
            SectionKind::WithBody(header, _body) => header,
            SectionKind::Error(_) => unreachable!("bodies cannot be added to Error sections"),
        };

        let inner = SectionKind::WithBody(header, Box::new(body));

        Section {
            inner,
            order: section.order,
        }
    }

    fn skip_if<F>(self, condition: F) -> Section
    where
        F: FnOnce() -> bool,
    {
        let mut section = Section::from(self);

        section.order = if condition() {
            Order::SkipEntirely
        } else {
            section.order
        };

        section
    }
}

impl<T> From<T> for Section
where
    T: Display + Send + Sync + 'static,
{
    fn from(header: T) -> Self {
        let inner = SectionKind::Header(Box::new(header));

        Self {
            inner,
            order: Order::BeforeSpanTrace,
        }
    }
}

impl fmt::Debug for Section {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.inner)
    }
}

impl fmt::Display for SectionKind {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            SectionKind::Header(header) => write!(f, "{}", header)?,
            SectionKind::WithBody(header, body) => {
                write!(f, "{}", header)?;
                writeln!(f)?;
                write!(
                    indenter::indented(f)
                        .with_format(indenter::Format::Uniform { indentation: "   " }),
                    "{}",
                    body
                )?;
            }
            SectionKind::Error(error) => {
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
            }
        }

        Ok(())
    }
}
