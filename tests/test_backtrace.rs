#[rustversion::not(nightly)]
#[ignore]
#[test]
fn test_backtrace() {}

#[rustversion::nightly]
#[test]
fn test_backtrace() {
    use eyre::eyre;

    let error = eyre!("oh no!");
    let _ = error.backtrace();
}
