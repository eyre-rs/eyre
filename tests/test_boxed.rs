use eyre::{eyre, ErrReport};
use std::error::Error as StdError;
use std::io;
use thiserror::Error;

#[derive(Error, Debug)]
#[error("outer")]
struct MyError {
    source: io::Error,
}

#[test]
fn test_boxed_str() {
    let error = Box::<dyn StdError + Send + Sync>::from("oh no!");
    let error: ErrReport = eyre!(error);
    assert_eq!("oh no!", error.to_string());
    assert_eq!(
        "oh no!",
        error
            .downcast_ref::<Box<dyn StdError + Send + Sync>>()
            .unwrap()
            .to_string()
    );
}

#[test]
fn test_boxed_thiserror() {
    let error = MyError {
        source: io::Error::new(io::ErrorKind::Other, "oh no!"),
    };
    let error = eyre!(error);
    assert_eq!("oh no!", error.source().unwrap().to_string());
}

#[test]
fn test_boxed_eyre() {
    let error = eyre!("oh no!").wrap_err("it failed");
    let error = eyre!(error);
    assert_eq!("oh no!", error.source().unwrap().to_string());
}
