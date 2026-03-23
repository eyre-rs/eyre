#![allow(dead_code)]

use eyre::{bail, set_hook, DefaultHandler, InstallError, Result};
use std::io;
use std::sync::OnceLock;

pub fn bail_literal() -> Result<()> {
    bail!("oh no!");
}

pub fn bail_fmt() -> Result<()> {
    bail!("{} {}!", "oh", "no");
}

pub fn bail_error() -> Result<()> {
    bail!(io::Error::other("oh no!"));
}

// Tests are multithreaded- use OnceLock to install hook once if auto-install
// feature is disabled.
pub fn maybe_install_handler() -> Result<(), InstallError> {
    static INSTALLER: OnceLock<Result<(), InstallError>> = OnceLock::new();

    if cfg!(not(feature = "auto-install")) {
        *INSTALLER.get_or_init(|| set_hook(Box::new(DefaultHandler::default_with)))
    } else {
        Ok(())
    }
}
