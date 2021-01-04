use std::panic::Location;

use eyre::WrapErr;

struct LocationHandler {
    actual: Option<&'static str>,
    expected: &'static str,
}

impl LocationHandler {
    fn new(expected: &'static str) -> Self {
        LocationHandler {
            actual: None,
            expected,
        }
    }
}

impl eyre::EyreHandler for LocationHandler {
    fn debug(
        &self,
        _error: &(dyn std::error::Error + 'static),
        _f: &mut std::fmt::Formatter<'_>,
    ) -> std::fmt::Result {
        assert_eq!(Some(self.expected), self.actual);
        Ok(())
    }

    fn track_caller(&mut self, location: &'static Location<'static>) {
        dbg!(location);
        self.actual = Some(location.file());
    }
}

#[test]
fn test_wrap_err() {
    let _ = eyre::set_hook(Box::new(|_e| {
        let expected_location = file!();
        Box::new(LocationHandler::new(expected_location))
    }));

    let err = std::fs::read_to_string("totally_fake_path")
        .wrap_err("oopsie")
        .unwrap_err();

    // should panic if the location isn't in our crate
    println!("{:?}", err);
}

#[test]
fn test_wrap_err_with() {
    let _ = eyre::set_hook(Box::new(|_e| {
        let expected_location = file!();
        Box::new(LocationHandler::new(expected_location))
    }));

    let err = std::fs::read_to_string("totally_fake_path")
        .wrap_err_with(|| "oopsie")
        .unwrap_err();

    // should panic if the location isn't in our crate
    println!("{:?}", err);
}

#[test]
fn test_context() {
    let _ = eyre::set_hook(Box::new(|_e| {
        let expected_location = file!();
        Box::new(LocationHandler::new(expected_location))
    }));

    let err = std::fs::read_to_string("totally_fake_path")
        .context("oopsie")
        .unwrap_err();

    // should panic if the location isn't in our crate
    println!("{:?}", err);
}

#[test]
fn test_with_context() {
    let _ = eyre::set_hook(Box::new(|_e| {
        let expected_location = file!();
        Box::new(LocationHandler::new(expected_location))
    }));

    let err = std::fs::read_to_string("totally_fake_path")
        .with_context(|| "oopsie")
        .unwrap_err();

    // should panic if the location isn't in our crate
    println!("{:?}", err);
}

#[test]
fn test_option_compat_wrap_err() {
    let _ = eyre::set_hook(Box::new(|_e| {
        let expected_location = file!();
        Box::new(LocationHandler::new(expected_location))
    }));

    use eyre::ContextCompat;
    let err = None::<()>.wrap_err("oopsie").unwrap_err();

    // should panic if the location isn't in our crate
    println!("{:?}", err);
}

#[test]
fn test_option_compat_wrap_err_with() {
    let _ = eyre::set_hook(Box::new(|_e| {
        let expected_location = file!();
        Box::new(LocationHandler::new(expected_location))
    }));

    use eyre::ContextCompat;
    let err = None::<()>.wrap_err_with(|| "oopsie").unwrap_err();

    // should panic if the location isn't in our crate
    println!("{:?}", err);
}

#[test]
fn test_option_compat_context() {
    let _ = eyre::set_hook(Box::new(|_e| {
        let expected_location = file!();
        Box::new(LocationHandler::new(expected_location))
    }));

    use eyre::ContextCompat;
    let err = None::<()>.context("oopsie").unwrap_err();

    // should panic if the location isn't in our crate
    println!("{:?}", err);
}

#[test]
fn test_option_compat_with_context() {
    let _ = eyre::set_hook(Box::new(|_e| {
        let expected_location = file!();
        Box::new(LocationHandler::new(expected_location))
    }));

    use eyre::ContextCompat;
    let err = None::<()>.with_context(|| "oopsie").unwrap_err();

    // should panic if the location isn't in our crate
    println!("{:?}", err);
}
