#[rustversion::not(nightly)]
#[ignore]
#[test]
fn test_backtrace() {}

#[rustversion::nightly]
#[test]
fn test_backtrace() {
    use eyre::{eyre, Report};

    let error: Report = eyre!("oh no!");
    let _ = error.backtrace();
}
