#[cfg(backtrace)]
pub use std::backtrace::Backtrace;

#[cfg(not(backtrace))]
#[derive(Debug)]
pub enum Backtrace {}

#[cfg(backtrace)]
macro_rules! backtrace_if_absent {
    ($err:expr) => {
        match $err.backtrace() {
            Some(_) => None,
            None => Some(Backtrace::capture().into()),
        }
    };
}

#[cfg(not(backtrace))]
macro_rules! backtrace_if_absent {
    ($err:expr) => {
        None
    };
}
