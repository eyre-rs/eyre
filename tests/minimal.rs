use color_eyre::{Help, Report};
use eyre::WrapErr;
use pretty_assertions::assert_eq;
use tracing::{info, instrument};

#[instrument]
#[test]
#[cfg(not(windows))]
fn minimal() -> Result<(), Report> {
    #[cfg(feature = "capture-spantrace")]
    install_tracing();

    let report = read_config().unwrap_err();
    let report = format!("Error: {:?}", report);

    println!("Expected\n{}", EXPECTED);
    println!("Actual\n{}", report);
    assert_eq!(EXPECTED, report);

    Ok(())
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
fn read_file(path: &str) -> Result<(), Report> {
    info!("Reading file");
    Ok(std::fs::read_to_string(path).map(drop)?)
}

#[instrument]
fn read_config() -> Result<(), Report> {
    read_file("fake_file")
        .wrap_err("Unable to read config")
        .suggestion("try using a file that exists next time")
}

// Define at the bottom to prevent it from changing line numbers
#[cfg(feature = "capture-spantrace")]
static EXPECTED: &str = "Error: 
   0: \u{1b}[31mUnable to read config\u{1b}[0m
   1: \u{1b}[31mNo such file or directory (os error 2)\u{1b}[0m

  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ SPANTRACE ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━
  
   0: \u{1b}[31mminimal\u{1b}[0m\u{1b}[31m::\u{1b}[0m\u{1b}[31mread_file\u{1b}[0m with \u{1b}[36mpath=\"fake_file\"\u{1b}[0m
      at tests/minimal.rs:41
   1: \u{1b}[31mminimal\u{1b}[0m\u{1b}[31m::\u{1b}[0m\u{1b}[31mread_config\u{1b}[0m
      at tests/minimal.rs:47

\u{1b}[36mSuggestion\u{1b}[0m: try using a file that exists next time";

#[cfg(not(feature = "capture-spantrace"))]
static EXPECTED: &str = "Error: 
   0: \u{1b}[31mUnable to read config\u{1b}[0m
   1: \u{1b}[31mNo such file or directory (os error 2)\u{1b}[0m

\u{1b}[36mSuggestion\u{1b}[0m: try using a file that exists next time";
