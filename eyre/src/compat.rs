use crate::{builder::ReportBuilder, Report};

/// Convert this result into an eyre [`Report`](crate::Report) result
///
/// This trait can also be used to provide conversions to eyre in `no-std` environments where
/// [`Error`](std::error::Error) is not yet available.
pub trait IntoEyre<T> {
    /// Convert this result into an eyre [`Report`](crate::Report) result
    fn into_eyre(self) -> crate::Result<T>;
}

/// See: [`IntoEyre`]
/// This is for crate authors to implement on their custom error types. Implementing this for your
/// Error type automatically implements `into_eyre` for `Result<T, E>`
pub trait IntoEyreReport {
    /// Convert this error into an eyre [`Report`](crate::Report)
    #[track_caller]
    fn into_eyre_report(self) -> Report;
}

impl<T, E> IntoEyre<T> for Result<T, E>
where
    E: IntoEyreReport,
{
    #[track_caller]
    fn into_eyre(self) -> crate::Result<T> {
        // Use a manual match to keep backtrace
        match self {
            Ok(v) => Ok(v),
            Err(err) => Err(err.into_eyre_report()),
        }
    }
}

#[cfg(feature = "anyhow-compat")]
impl IntoEyreReport for anyhow::Error {
    #[track_caller]
    fn into_eyre_report(self) -> Report
    where
        Self: Sized,
    {
        let report = ReportBuilder::default()
            .with_backtrace(self.backtrace())
            .from_boxed(self.into());

        report
    }
}
