#![allow(dead_code, unused_imports)]
use color_eyre::eyre;
use eyre::{Report, Result};
use tracing::instrument;

#[instrument]
#[cfg(feature = "issue-url")]
fn main() -> Result<(), Report> {
    #[cfg(feature = "capture-spantrace")]
    install_tracing();

    color_eyre::config::HookBuilder::default()
        .issue_url("https://github.com/yaahc/jane-eyre/issues/new")
        .add_issue_metadata("version", "0.1.0")
        .install()?;

    let report = read_config().unwrap_err();
    eprintln!("Error: {:?}", report);

    read_config2();

    Ok(())
}

#[cfg(not(feature = "issue-url"))]
fn main() {
    unimplemented!("this example requires the \"issue-url\" feature")
}

#[cfg(feature = "capture-spantrace")]
fn install_tracing() {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};

    let fmt_layer = fmt::layer().with_target(false);
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();
}

#[instrument]
fn read_file(path: &str) -> Result<String> {
    Ok(std::fs::read_to_string(path)?)
}

#[instrument]
fn read_config() -> Result<()> {
    read_file("fake_file")?;

    Ok(())
}

#[instrument]
fn read_file2(path: &str) {
    if let Err(e) = std::fs::read_to_string(path) {
        panic!("{}", e);
    }
}

#[instrument]
fn read_config2() {
    read_file2("fake_file")
}
