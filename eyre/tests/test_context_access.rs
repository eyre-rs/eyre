#![cfg(feature = "anyhow")]

mod common;

use crate::common::maybe_install_handler;

#[test]
fn test_context() {
    use eyre::{report, Report};

    maybe_install_handler().unwrap();

    let error: Report = report!("oh no!");
    let _ = error.context();
}
