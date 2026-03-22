use color_eyre::eyre;
use eyre::report;

#[test]
fn disabled() {
    color_eyre::config::HookBuilder::default()
        .display_env_section(false)
        .install()
        .unwrap();

    let report = report!("error occured");

    let report = format!("{report:?}");
    assert!(!report.contains("RUST_BACKTRACE"));
}
