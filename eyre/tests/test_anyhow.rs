#![cfg(feature = "anyhow")]

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

fn anyhow_fail() -> anyhow::Result<()> {
    use anyhow::Context;
    Err(RootError).context("Ouch!").context("Anyhow context A")
}

fn eyre_fail() -> eyre::Result<()> {
    use eyre::Context;
    Err(RootError).wrap_err("Ouch!").wrap_err("Eyre context A")
}

fn eyre_calling_anyhow() -> eyre::Result<()> {
    use anyhow::Context;
    anyhow_fail().context("Anyhow context B")?;
    Ok(())
}

fn anyhow_calling_eyre() -> anyhow::Result<()> {
    use eyre::Context;
    eyre_fail().wrap_err("Eyre context B")?;
    Ok(())
}

#[test]
fn test_anyhow_conversion() {
    use eyre::Context;
    let error: eyre::Report = eyre_calling_anyhow().wrap_err("Eyre context").unwrap_err();

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
}

#[test]
fn test_eyre_conversion() {
    use anyhow::Context;
    let error: anyhow::Error = anyhow_calling_eyre().context("Anyhow context").unwrap_err();

    let chain = error.chain().map(ToString::to_string).collect::<Vec<_>>();
    assert_eq!(
        chain,
        [
            "Anyhow context",
            // Eyre context
            "Eyre context B",
            "Eyre context A",
            // Eyre error
            "Ouch!",
            // Original concrete error, shows up in chain too
            "RootError"
        ]
    );
}
