#[rustversion::not(nightly)]
#[ignore]
#[test]
fn test_backtrace() {}

#[rustversion::nightly]
#[test]
fn test_backtrace() {
    use eyre::{eyre, ErrReport};

    let error: ErrReport = eyre!("oh no!");
    let _ = error.backtrace();
}
