Simple-Eyre
===========

[![Latest Version](https://img.shields.io/crates/v/simple-eyre.svg)](https://crates.io/crates/simple-eyre)
[![Rust Documentation](https://img.shields.io/badge/api-rustdoc-blue.svg)](https://docs.rs/simple-eyre)

```toml
[dependencies]
simple-eyre = "0.1"
```

<br>

## Example

```rust
fn eyre::ErrReport;

fn find_git_root() -> Result<PathBuf, ErrReport> {
    find_dir_in_ancestors(".git")?;
}
```

<br>

## Details

- This library is meant to be used as a minimal example of how to use
  `eyre-impl`. It implements the absolute minimum necessary to function as a
  dynamic error wrapper that associates some context with it. In this case the
  context is only a Backtrace.

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




