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

fn bubble() -> eyre::Result<()> {
    use anyhow::Context;
    this_function_fails().context("Anyhow context B")?;

    Ok(())
}

#[test]
fn anyhow_conversion() {
    use eyre::WrapErr;
    let error: Report = bubble().wrap_err("Eyre context").unwrap_err();

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

    // let error = Report::msg("A").wrap_err("B").wrap_err("C");
}
