#[cfg(feature = "track-caller")]
#[test]
fn disabled() {
    use color_eyre::eyre;
    use eyre::report;

    color_eyre::config::HookBuilder::default()
        .display_location_section(false)
        .install()
        .unwrap();

    let report = report!("error occured");

    let report = format!("{report:?}");
    assert!(!report.contains("Location:"));
}
