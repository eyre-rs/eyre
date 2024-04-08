## simple-eyre

[![Latest Version](https://img.shields.io/crates/v/simple-eyre.svg)](https://crates.io/crates/simple-eyre)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/simple-eyre)

This library provides a custom [`eyre::EyreHandler`] type for usage with [`eyre`] that provides
a minimal error report with no additional context. Essentially the minimal implementation of an
error reporter.

## Setup

Add the following to your toml file:

```toml
[dependencies]
simple-eyre = "0.3"
```

Then install the hook handler before constructing any `eyre::Report` types.

# Example

```rust,should_panic
use simple_eyre::eyre::{eyre, WrapErr, Report};

fn main() -> Result<(), Report> {
    simple_eyre::install()?;

    let e: Report = eyre!("oh no this program is just bad!");

    Err(e).wrap_err("usage example successfully experienced a failure")
}
```

[`eyre::EyreHandler`]: https://docs.rs/eyre/*/eyre/trait.EyreHandler.html
[`eyre`]: https://docs.rs/eyre

#### License

<sup>
Licensed under either of <a href="LICENSE-APACHE">Apache License, Version
2.0</a> or <a href="LICENSE-MIT">MIT license</a> at your option.
</sup>

<br>

<sub>
Unless you explicitly state otherwise, any contribution intentionally submitted
for inclusion in this crate by you, as defined in the Apache-2.0 license, shall
be dual licensed as above, without any additional terms or conditions.
</sub>
