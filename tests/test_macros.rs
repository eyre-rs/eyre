#![allow(clippy::eq_op)]
mod common;

use self::common::*;
use eyre::{ensure, eyre, Result};
use std::cell::Cell;
use std::future;

#[test]
fn test_messages() {
    assert_eq!("oh no!", bail_literal().unwrap_err().to_string());
    assert_eq!("oh no!", bail_fmt().unwrap_err().to_string());
    assert_eq!("oh no!", bail_error().unwrap_err().to_string());
}

#[test]
fn test_ensure() {
    let f = || -> Result<()> {
        ensure!(1 + 1 == 2, "This is correct");
        Ok(())
    };
    assert!(f().is_ok());

    let v = 1;
    let f = || -> Result<()> {
        ensure!(v + v == 2, "This is correct, v: {}", v);
        Ok(())
    };
    assert!(f().is_ok());

    let f = || -> Result<()> {
        ensure!(v + v == 1, "This is not correct, v: {}", v);
        Ok(())
    };
    assert!(f().is_err());
}

#[test]
fn test_temporaries() {
    fn require_send_sync(_: impl Send + Sync) {}

    require_send_sync(async {
        // If eyre hasn't dropped any temporary format_args it creates by the
        // time it's done evaluating, those will stick around until the
        // semicolon, which is on the other side of the await point, making the
        // enclosing future non-Send.
        future::ready(eyre!("...")).await;
    });

    fn message(cell: Cell<&str>) -> &str {
        cell.get()
    }

    require_send_sync(async {
        future::ready(eyre!(message(Cell::new("...")))).await;
    });
}

#[test]
fn test_capture_format_args() {
    let var = 42;
    let err = eyre!("interpolate {var}");
    assert_eq!("interpolate 42", err.to_string());
}

#[test]
fn test_brace_escape() {
    let err = eyre!("unterminated ${{..}} expression");
    assert_eq!("unterminated ${..} expression", err.to_string());
}
