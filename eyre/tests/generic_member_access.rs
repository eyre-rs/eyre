#![cfg_attr(generic_member_access, feature(error_generic_member_access))]

mod common;

#[cfg(all(generic_member_access, not(miri)))]
#[test]
/// Tests that generic member access works through an `eyre::Report`
fn generic_member_access() {
    use crate::common::maybe_install_handler;

    use eyre::WrapErr;
    use std::backtrace::Backtrace;
    use std::fmt::Display;

    fn fail() -> Result<(), MyError> {
        Err(MyError {
            cupcake: MyCupcake("Blueberry".into()),
            backtrace: std::backtrace::Backtrace::capture(),
        })
    }

    maybe_install_handler().unwrap();

    std::env::set_var("RUST_BACKTRACE", "1");

    #[derive(Debug, PartialEq)]
    struct MyCupcake(String);

    #[derive(Debug)]
    struct MyError {
        cupcake: MyCupcake,
        backtrace: std::backtrace::Backtrace,
    }

    impl Display for MyError {
        fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
            write!(f, "Error: {}", self.cupcake.0)
        }
    }

    impl std::error::Error for MyError {
        fn provide<'a>(&'a self, request: &mut std::error::Request<'a>) {
            request
                .provide_ref(&self.cupcake)
                .provide_ref(&self.backtrace);
        }
    }

    let err = fail()
        .wrap_err("Failed to bake my favorite cupcake")
        .unwrap_err();

    let err: Box<dyn std::error::Error> = err.into();

    assert!(
        format!("{:?}", err).contains("generic_member_access::generic_member_access::fail"),
        "should contain the source error backtrace"
    );

    assert_eq!(
        std::error::request_ref::<MyCupcake>(&*err),
        Some(&MyCupcake("Blueberry".into()))
    );

    let bt = std::error::request_ref::<Backtrace>(&*err).unwrap();

    assert!(
        bt.to_string()
            .contains("generic_member_access::generic_member_access::fail"),
        "should contain the fail method as it was captured by the original error\n\n{}",
        bt
    );
}
