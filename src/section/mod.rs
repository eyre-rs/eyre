//! Helpers for adding custom sections to error reports
use std::fmt::{self, Display, Write};

pub mod help;

/// An indenteted section with a header for an error report
///
/// # Details
///
/// This helper provides two functions to help with constructing nicely formatted
/// error reports. First, it handles indentation of every line of the body for
/// you, and makes sure it is consistent with the rest of color-eyre's output.
/// Second, it omits outputting the header if the body itself is empty,
/// preventing unnecessary pollution of the report for sections with dynamic
/// content.
///
/// # Examples
///
/// ```rust
/// use color_eyre::{eyre::eyre, SectionExt, Help, Report};
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
///                 .with_section(move || stdout.trim().to_string().header("Stdout:"))
///                 .with_section(move || stderr.trim().to_string().header("Stderr:"))
///         } else {
///             Ok(stdout.into())
///         }
///     }
/// }
/// ```
#[allow(missing_debug_implementations)]
pub struct Section<H, B> {
    header: H,
    body: B,
}

/// Extension trait for constructing sections with commonly used formats
pub trait SectionExt: Sized {
    /// Add a header to a `Section` and indent the body
    ///
    /// Bodies are always indented to the same level as error messages and spans.
    /// The header is not printed if the display impl of the body produces no
    /// output.
    ///
    /// # Examples
    ///
    /// ```rust
    /// use color_eyre::{eyre::eyre, Help, SectionExt, Report};
    ///
    /// let all_in_header = "header\n   body\n   body";
    /// let report = Err::<(), Report>(eyre!("an error occurred"))
    ///     .section(all_in_header)
    ///     .unwrap_err();
    ///
    /// let just_header = "header";
    /// let just_body = "body\nbody";
    /// let report2 = Err::<(), Report>(eyre!("an error occurred"))
    ///     .section(just_body.header(just_header))
    ///     .unwrap_err();
    ///
    /// assert_eq!(format!("{:?}", report), format!("{:?}", report2))
    /// ```
    fn header<C>(self, header: C) -> Section<C, Self>
    where
        C: Display + Send + Sync + 'static;
}

impl<T> SectionExt for T
where
    T: Display + Send + Sync + 'static,
{
    fn header<C>(self, header: C) -> Section<C, Self>
    where
        C: Display + Send + Sync + 'static,
    {
        Section { body: self, header }
    }
}

impl<H, B> fmt::Display for Section<H, B>
where
    H: Display + Send + Sync + 'static,
    B: Display + Send + Sync + 'static,
{
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        let mut headered = crate::writers::HeaderWriter {
            inner: f,
            header: &self.header,
            started: false,
        };

        let mut headered = crate::writers::HeaderWriter {
            inner: headered.ready(),
            header: &"\n",
            started: false,
        };

        let mut headered = headered.ready();

        let mut indented = indenter::indented(&mut headered)
            .with_format(indenter::Format::Uniform { indentation: "   " });

        write!(&mut indented, "{}", self.body)?;

        Ok(())
    }
}
