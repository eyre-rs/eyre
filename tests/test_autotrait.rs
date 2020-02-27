use eyre::ErrReport;

#[test]
fn test_send() {
    fn assert_send<T: Send>() {}
    assert_send::<ErrReport>();
}

#[test]
fn test_sync() {
    fn assert_sync<T: Sync>() {}
    assert_sync::<ErrReport>();
}
