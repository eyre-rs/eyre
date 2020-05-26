## color-eyre

[![Build Status][actions-badge]][actions-url]
[![Latest Version][version-badge]][version-url]
[![Rust Documentation][docs-badge]][docs-url]

[actions-badge]: https://github.com/yaahc/color-eyre/workflows/Continuous%20integration/badge.svg
[actions-url]: https://github.com/yaahc/color-eyre/actions?query=workflow%3A%22Continuous+integration%22
[version-badge]: https://img.shields.io/crates/v/color-eyre.svg
[version-url]: https://crates.io/crates/color-eyre
[docs-badge]: https://img.shields.io/badge/docs-latest-blue.svg
[docs-url]: https://docs.rs/color-eyre

A custom context for the [`eyre`] crate for colorful error reports with suggestions, custom
sections, [`tracing-error`] support, and backtraces on stable.

## TLDR

`color_eyre` helps you build error reports that look like this:

![custom section example](https://raw.githubusercontent.com/yaahc/color-eyre/master/pictures/custom_section.png)

## Setup

Add the following to your toml file:

```toml
[dependencies]
eyre = "0.4"
color-eyre = "0.3"
```

And then import the type alias from color-eyre for [`eyre::Report`] or [`eyre::Result`].

```rust
use color_eyre::Report;

// or

fn example() -> color_eyre::Result<()> {
    # Ok(())
    // ...
}
```

### Disabling tracing support

If you don't plan on using `tracing_error` and `SpanTrace` you can disable the
tracing integration to cut down on unused dependencies:

```toml
[dependencies]
eyre = "0.4"
color-eyre = { version = "0.3", default-features = false }
```

## Features

### Multiple report format verbosity levels

`color-eyre` provides 3 different report formats for how it formats the captured `SpanTrace`
and `Backtrace`, minimal, short, and full. Take the following example, taken from
[`examples/usage.rs`]:

```rust
use color_eyre::{Help, Report};
use eyre::WrapErr;
use tracing::{info, instrument};

#[instrument]
fn main() -> Result<(), Report> {
    #[cfg(feature = "capture-spantrace")]
    install_tracing();

    Ok(read_config()?)
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

#[instrument]
fn read_file(path: &str) -> Result<(), Report> {
    info!("Reading file");
    Ok(std::fs::read_to_string(path).map(drop)?)
}

#[instrument]
fn read_config() -> Result<(), Report> {
    read_file("fake_file")
        .wrap_err("Unable to read config")
        .suggestion("try using a file that exists next time")
}
```

---

Running `cargo run --example usage` without `RUST_LIB_BACKTRACE` set will produce a minimal
report like this:

![minimal report format](https://raw.githubusercontent.com/yaahc/color-eyre/master/pictures/minimal.png)

<br>

Running `RUST_LIB_BACKTRACE=1 cargo run --example usage` tells `color-eyre` to use the short
format, which additionally capture a [`backtrace::Backtrace`]:

![short report format](https://raw.githubusercontent.com/yaahc/color-eyre/master/pictures/short.png)

<br>

Finally, running `RUST_LIB_BACKTRACE=full cargo run --example usage` tells `color-eyre` to use
the full format, which in addition to the above will attempt to include source lines where the
error originated from, assuming it can find them on the disk.

![full report format](https://raw.githubusercontent.com/yaahc/color-eyre/master/pictures/full.png)

### Custom `Section`s for error reports via [`Help`] trait

The `section` module provides helpers for adding extra sections to error reports. Sections are
disinct from error messages and are displayed independently from the chain of errors. Take this
example of adding sections to contain `stderr` and `stdout` from a failed command, taken from
[`examples/custom_section.rs`]:

```rust
use color_eyre::{SectionExt, Help, Report};
use eyre::eyre;
use std::process::Command;
use tracing::instrument;

trait Output {
    fn output2(&mut self) -> Result<String, Report>;
}

impl Output for Command {
    #[instrument]
    fn output2(&mut self) -> Result<String, Report> {
        let output = self.output()?;

        let stdout = String::from_utf8_lossy(&output.stdout);

        if !output.status.success() {
            let stderr = String::from_utf8_lossy(&output.stderr);
            Err(eyre!("cmd exited with non-zero status code"))
                .with_section(move || {
                    "Stdout:"
                        .skip_if(|| stdout.is_empty())
                        .body(stdout.trim().to_string())
                })
                .with_section(move || {
                    "Stderr:"
                        .skip_if(|| stderr.is_empty())
                        .body(stderr.trim().to_string())
                })
        } else {
            Ok(stdout.into())
        }
    }
}
```

---

Here we have an function that, if the command exits unsuccessfully, creates a report indicating
the failure and attaches two sections, one for `stdout` and one for `stderr`. Each section
includes a short header and a body that contains the actual output. Additionally these sections
use `skip_if` to tell the report not to include them if there was no output, preventing empty
sections from polluting the end report.

Running `cargo run --example custom_section` shows us how these sections are included in the
output:

![custom section example](https://raw.githubusercontent.com/yaahc/color-eyre/master/pictures/custom_section.png)

Only the `Stderr:` section actually gets included. The `cat` command fails, so stdout ends up
being empty and is skipped in the final report. This gives us a short and concise error report
indicating exactly what was attempted and how it failed.

### Aggregating multiple errors into one report

It's not uncommon for programs like batched task runners or parsers to want to
return an error with multiple sources. The current version of the error trait
does not support this use case very well, though there is [work being
done](https://github.com/rust-lang/rfcs/pull/2895) to improve this.

For now however one way to work around this is to compose errors outside the
error trait. `color-eyre` supports such composition in its error reports via
the `Help` trait.

For an example of how to aggregate errors check out [`examples/multiple_errors.rs`].

### Custom configuration for `color-backtrace` for setting custom filters and more

The pretty printing for backtraces and span traces isn't actually provided by `color-eyre`, but
instead comes from its dependencies [`color-backtrace`] and [`color-spantrace`].
`color-backtrace` in particular has many more features than are exported by `color-eyre`, such
as customized color schemes, panic hooks, and custom frame filters. The custom frame filters
are particularly useful when combined with `color-eyre`, so to enable their usage we provide
the `install` fn for setting up a custom `BacktracePrinter` with custom filters installed.

For an example of how to setup custom filters, check out [`examples/custom_filter.rs`].

## Explanation

This crate works by defining a `Context` type which implements [`eyre::EyreContext`]
and a pair of type aliases for setting this context type as the parameter of
[`eyre::Report`].

```rust
use color_eyre::Context;

pub type Report = eyre::Report<Context>;
pub type Result<T, E = Report> = core::result::Result<T, E>;
```

Please refer to the [`Context`] type's docs for more details about its feature set.

[`eyre`]: https://docs.rs/eyre
[`tracing-error`]: https://docs.rs/tracing-error
[`color-backtrace`]: https://docs.rs/color-backtrace
[`eyre::EyreContext`]: https://docs.rs/eyre/*/eyre/trait.EyreContext.html
[`backtrace::Backtrace`]: https://docs.rs/backtrace/*/backtrace/struct.Backtrace.html
[`tracing_error::SpanTrace`]: https://docs.rs/tracing-error/*/tracing_error/struct.SpanTrace.html
[`color-spantrace`]: https://github.com/yaahc/color-spantrace
[`Help`]: https://docs.rs/color-eyre/*/color_eyre/trait.Help.html
[`eyre::Report`]: https://docs.rs/eyre/*/eyre/struct.Report.html
[`eyre::Result`]: https://docs.rs/eyre/*/eyre/type.Result.html
[`Context`]: https://docs.rs/color-eyre/*/color_eyre/struct.Context.html
[`examples/usage.rs`]: https://github.com/yaahc/color-eyre/blob/master/examples/usage.rs
[`examples/custom_filter.rs`]: https://github.com/yaahc/color-eyre/blob/master/examples/custom_filter.rs
[`examples/custom_section.rs`]: https://github.com/yaahc/color-eyre/blob/master/examples/custom_section.rs
[`examples/multiple_errors.rs`]: https://github.com/yaahc/color-eyre/blob/master/examples/multiple_errors.rs

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
