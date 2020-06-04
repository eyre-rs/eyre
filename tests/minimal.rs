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

#[instrument]
#[test]
#[cfg(windows)]
fn minimal() -> Result<(), Report> {
    #[cfg(feature = "capture-spantrace")]
    install_tracing();

    let report = read_config().unwrap_err();
    let report = format!("Error: {:?}", report);

    println!("Expected\n{}", WINDOWS_EXPECTED);
    println!("Actual\n{}", report);
    assert_eq!(WINDOWS_EXPECTED, report);

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
#[cfg(not(windows))]
#[cfg(feature = "capture-spantrace")]
static EXPECTED: &str = "Error: \n   0: \u{1b}[38;5;9mUnable to read config\u{1b}[0m\n   1: \u{1b}[38;5;9mNo such file or directory (os error 2)\u{1b}[0m\n\n  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ SPANTRACE ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  \n   0: \u{1b}[38;5;9mminimal\u{1b}[0m\u{1b}[38;5;9m::\u{1b}[0m\u{1b}[38;5;9mread_file\u{1b}[0m with \u{1b}[38;5;14mpath=\"fake_file\"\u{1b}[0m\n      at tests/minimal.rs:58\n   1: \u{1b}[38;5;9mminimal\u{1b}[0m\u{1b}[38;5;9m::\u{1b}[0m\u{1b}[38;5;9mread_config\u{1b}[0m\n      at tests/minimal.rs:64\n\n\u{1b}[38;5;14mSuggestion\u{1b}[0m: try using a file that exists next time";

#[cfg(not(windows))]
#[cfg(not(feature = "capture-spantrace"))]
static EXPECTED: &str = "Error: \n   0: \u{1b}[38;5;9mUnable to read config\u{1b}[0m\n   1: \u{1b}[38;5;9mNo such file or directory (os error 2)\u{1b}[0m\n\n\u{1b}[38;5;14mSuggestion\u{1b}[0m: try using a file that exists next time";

#[cfg(windows)]
#[cfg(feature = "capture-spantrace")]
static WINDOWS_EXPECTED: &str = "Error: \n   0: \u{1b}[38;5;9mUnable to read config\u{1b}[0m\n   1: \u{1b}[38;5;9mThe system cannot find the file specified. (os error 2)\u{1b}[0m\n\n  ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━ SPANTRACE ━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━\n  \n   0: \u{1b}[38;5;9mminimal\u{1b}[0m\u{1b}[38;5;9m::\u{1b}[0m\u{1b}[38;5;9mread_file\u{1b}[0m with \u{1b}[38;5;14mpath=\"fake_file\"\u{1b}[0m\n      at tests\\minimal.rs:58\n   1: \u{1b}[38;5;9mminimal\u{1b}[0m\u{1b}[38;5;9m::\u{1b}[0m\u{1b}[38;5;9mread_config\u{1b}[0m\n      at tests\\minimal.rs:64\n\n\u{1b}[38;5;14mSuggestion\u{1b}[0m: try using a file that exists next time";

#[cfg(windows)]
#[cfg(not(feature = "capture-spantrace"))]
static WINDOWS_EXPECTED: &str = "Error: \n   0: \u{1b}[38;5;9mUnable to read config\u{1b}[0m\n   1: \u{1b}[38;5;9mThe system cannot find the file specified. (os error 2)\u{1b}[0m\n\n\u{1b}[38;5;14mSuggestion\u{1b}[0m: try using a file that exists next time";
