Simple-Eyre
===========

[![Latest Version](https://img.shields.io/crates/v/simple-eyre.svg)](https://crates.io/crates/simple-eyre)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/simple-eyre)

This library provides a custom [`eyre::EyreContext`] type for usage with
[`eyre`] that provides a minimal error report with no additional context.
Essentially the minimal implementation of an error reporter.

```toml
[dependencies]
eyre = "0.4"
simple-eyre = "0.2"
```

<br>

## Example

```rust
use eyre::{eyre, WrapErr};
use simple_eyre::Report;

fn main() -> Result<(), Report> {
    let e: Report = eyre!("oh no this program is just bad!");

    Err(e).wrap_err("usage example successfully experienced a failure")
}
```

<br>

<br>

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

[`eyre::EyreContext`]: https://docs.rs/eyre/*/eyre/trait.EyreContext.html
[`eyre`]: https://docs.rs/eyre
