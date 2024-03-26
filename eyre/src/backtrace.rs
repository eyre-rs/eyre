#[cfg(backtrace)]
pub(crate) use std::backtrace::Backtrace;

#[cfg(not(backtrace))]
pub(crate) enum Backtrace {}

#[cfg(backtrace)]
macro_rules! capture_backtrace {
    () => {
        Some(Backtrace::capture())
    };
}

#[cfg(not(backtrace))]
macro_rules! capture_backtrace {
    () => {
        None
    };
}
/// Capture a backtrace iff there is not already a backtrace in the error chain
#[cfg(generic_member_access)]
macro_rules! backtrace_if_absent {
    ($err:expr) => {
        match std::error::request_ref::<std::backtrace::Backtrace>($err as &dyn std::error::Error) {
            Some(_) => None,
            None => capture_backtrace!(),
        }
    };
}

#[cfg(not(generic_member_access))]
macro_rules! backtrace_if_absent {
    ($err:expr) => {
        capture_backtrace!()
    };
}
