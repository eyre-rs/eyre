use color_eyre::eyre;
use eyre::eyre;

fn foo5() -> eyre::Report {
    foo4()
}
fn foo4() -> eyre::Report {
    foo3()
}
fn foo3() -> eyre::Report {
    foo2()
}
fn foo2() -> eyre::Report {
    foo1()
}
fn foo1() -> eyre::Report {
    eyre::eyre!("error occured")
}

#[test]
fn stacktrace_forward() {
    color_eyre::config::HookBuilder::default()
        .display_env_section(true)
        .reversed_stacktrace(false)
        .install()
        .unwrap();

    let report = foo5();

    let report = format!("{:?}", report);
    println!("{}", report);
    assert!(report.contains("RUST_BACKTRACE"));
}

#[test]
fn stacktrace_reversed() {
    color_eyre::config::HookBuilder::default()
        .display_env_section(true)
        .reversed_stacktrace(true)
        .install()
        .unwrap();

    let report = foo5();

    let report = format!("{:?}", report);
    println!("{}", report);
    assert!(report.contains("RUST_BACKTRACE"));
}
