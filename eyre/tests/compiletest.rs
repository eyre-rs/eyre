#[cfg_attr(not(backtrace), ignore)]
#[cfg_attr(miri, ignore)]
#[test]
fn ui() {
    let mut test_dir = std::path::PathBuf::from(std::env::var("CARGO_MANIFEST_DIR").unwrap());
    test_dir.push("tests");
    test_dir.push("ui");
    let rust_backtrace_val = "1";
    let mut config = ui_test::Config {
        mode: ui_test::Mode::Run { exit_code: 0 },
        filter_files: vec!["ui_test".to_owned()],

        ..ui_test::Config::cargo(test_dir)
    };
    config.program.args = vec!["run".into()];
    config.program.envs = vec![("RUST_BACKTRACE".into(), Some(rust_backtrace_val.into()))];

    let _ = ui_test::run_tests(config);
}
