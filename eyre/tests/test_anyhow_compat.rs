use anyhow::Context;
use eyre::WrapErr;
use eyre::{compat::IntoEyre, Report};

fn this_function_fails() -> anyhow::Result<()> {
    anyhow::bail!("Ouch!")
}

fn bubble() -> eyre::Result<()> {
    this_function_fails()
        .context("Anyhow::context")
        .into_eyre()
        .wrap_err("Eyre::wrap_err")?;

    Ok(())
}

#[test]
fn anyhow_conversion() {
    let error: Report = bubble().unwrap_err();
    // let error = Report::msg("A").wrap_err("B").wrap_err("C");

    eprintln!("{error:?}");
}
