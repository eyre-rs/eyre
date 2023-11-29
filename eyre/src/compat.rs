use crate::Report;

/// Convert this result into an eyre [`Report`](crate::Report) result
///
/// This trait can also be used to provide conversions to eyre in `no-std` environments where
/// [`Error`](std::error::Error) is not yet available.
pub trait IntoEyre<T> {
    /// Convert this result into an eyre [`Report`](crate::Report) result
    fn into_eyre(self) -> crate::Result<T>
    where
        Self: Sized,
    {
        todo!()
    }
}

/// See: [`IntoEyre`]
/// This is for crate authors to implement on their custom error types. Implementing this for your
/// Error type automatically implements `into_eyre` for `Result<T, E>`
pub trait IntoEyreReport {
    /// Convert this error into an eyre [`Report`](crate::Report)
    fn into_eyre_report(self) -> Report
    where
        Self: Sized,
    {
        todo!()
    }
}

impl<T, E> IntoEyre<T> for Result<T, E>
where
    E: IntoEyreReport,
{
    fn into_eyre(self) -> crate::Result<T> {
        self.map_err(E::into_eyre_report)
    }
}

#[cfg(feature = "anyhow-compat")]
impl IntoEyreReport for anyhow::Error {
    fn into_eyre_report(self) -> Report
    where
        Self: Sized,
    {
        // dbg!(
        //     self.root_cause(),
        //     self.source(),
        //     self.chain().rev().collect::<Vec<_>>(),
        //     self.chain()
        //         .rev()
        //         .map(|v| v.to_string())
        //         .collect::<Vec<_>>()
        // );

        let mut chain = self.chain().rev();

        // We can't store the actual error
        // PENDING: https://github.com/dtolnay/anyhow/issues/327
        let head = chain
            .next()
            .expect("Error chain contains at least one error");

        let report = Report::msg(head.to_string());
        // chai
        // eprintln!("{:?}", chain.map(|v| v.to_string()).collect::<Vec<_>>());

        // report

        chain.fold(report, |report, err| {
            // We can't write the actual error
            // PENDING: https://github.com/dtolnay/anyhow/issues/327
            report.wrap_err(err.to_string())
        })
    }
}
