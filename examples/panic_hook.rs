use tracing::instrument;

#[instrument]
fn main() {
    #[cfg(feature = "capture-spantrace")]
    install_tracing();
    color_eyre::install().unwrap();

    do_thing(42);
}

#[instrument]
fn do_thing(thing: u32) {
    panic!("some real basic stuff went wrong")
}

#[cfg(feature = "capture-spantrace")]
fn install_tracing() {
    use tracing_error::ErrorLayer;
    use tracing_subscriber::prelude::*;
    use tracing_subscriber::{fmt, EnvFilter};

    let fmt_layer = fmt::layer().with_target(false);
    let filter_layer = EnvFilter::try_from_default_env()
        .or_else(|_| EnvFilter::try_new("info"))
        .unwrap();

    tracing_subscriber::registry()
        .with(filter_layer)
        .with(fmt_layer)
        .with(ErrorLayer::default())
        .init();
}
