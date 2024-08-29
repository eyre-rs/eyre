#![cfg(generic_member_access)]
#![feature(error_generic_member_access)]

use eyre::Report;
use std::fmt::Display;

#[derive(Debug)]
struct RootError;

impl Display for RootError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "RootError")
    }
}

impl std::error::Error for RootError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        None
    }
}

fn this_function_fails() -> anyhow::Result<()> {
    use anyhow::Context;

    Err(RootError).context("Ouch!").context("Anyhow context A")
}

fn test_failure() -> eyre::Result<()> {
    use anyhow::Context;
    this_function_fails().context("Anyhow context B")?;

    Ok(())
}

#[test]
fn anyhow_conversion() {
    use eyre::WrapErr;
    let error: Report = test_failure().wrap_err("Eyre context").unwrap_err();

    eprintln!("Error: {:?}", error);

    let chain = error.chain().map(ToString::to_string).collect::<Vec<_>>();
    assert_eq!(
        chain,
        [
            "Eyre context",
            // Anyhow context
            "Anyhow context B",
            "Anyhow context A",
            // Anyhow error
            "Ouch!",
            // Original concrete error, shows up in chain too
            "RootError"
        ]
    );

    let backtrace = std::error::request_ref::<std::backtrace::Backtrace>(&*error).unwrap();
    dbg!(backtrace);
}
