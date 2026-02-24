# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## [Unreleased] - ReleaseDate

## [0.3.2](https://github.com/eyre-rs/eyre/compare/simple-eyre-v0.3.1...simple-eyre-v0.3.2) - 2026-02-24

### Added

- introduce an `"anyhow"` compatibility layer feature flag ([#138](https://github.com/eyre-rs/eyre/pull/138))

### Fixed

- remove duplicate thiserror reference
- compile probe and nightly backtraces
- fixed minor errors in README, ?enhanced? punny-ness ([#33](https://github.com/eyre-rs/eyre/pull/33))
- fix type inference issue in eyre macro autoderef behavior ([#27](https://github.com/eyre-rs/eyre/pull/27))

### Other

- Add simple-eyre to workspace
- Add 'simple-eyre/' from commit 'bcfff0f56f278dca96cdd45de0e741227f15f3b0'
- Merge branch 'master' into color-eyre
- Extend `Option` with `ok_or_eyre` ([#129](https://github.com/eyre-rs/eyre/pull/129))
- Update documentation on no_std support. ([#111](https://github.com/eyre-rs/eyre/pull/111))
- Move eyre code into a folder ([#107](https://github.com/eyre-rs/eyre/pull/107))
- Grammer/typo ([#85](https://github.com/eyre-rs/eyre/pull/85))
- Add community discord to readme ([#76](https://github.com/eyre-rs/eyre/pull/76))
- Release 0.6.8 ([#74](https://github.com/eyre-rs/eyre/pull/74))
- Release 0.6.6 ([#67](https://github.com/eyre-rs/eyre/pull/67))
- Fix typos. ([#39](https://github.com/eyre-rs/eyre/pull/39))
- Switch report handler to a global hook ([#29](https://github.com/eyre-rs/eyre/pull/29))
- Rename EyreContext to EyreHandler ([#26](https://github.com/eyre-rs/eyre/pull/26))
- Fixed typo ([#23](https://github.com/eyre-rs/eyre/pull/23))
- Remove unwanted apostrophe from “its ability” ([#21](https://github.com/eyre-rs/eyre/pull/21))
- bump indenter dep version and update readme
- Add `ContextCompat` trait for porting from `anyhow` ([#15](https://github.com/eyre-rs/eyre/pull/15))
- remove member access fns and backtrace fn ([#14](https://github.com/eyre-rs/eyre/pull/14))
- simplify docs and add back compat support ([#12](https://github.com/eyre-rs/eyre/pull/12))
- Fix strings on last few examples ([#11](https://github.com/eyre-rs/eyre/pull/11))
- merge upstream changes from anyhow ([#10](https://github.com/eyre-rs/eyre/pull/10))
- Rename ErrReport to Report ([#9](https://github.com/eyre-rs/eyre/pull/9))
- provide a default impl for the Display trait
- Fix tracing_error::SpanTrace link in Readme 
- Update readme
- More readme / docs updates
- More updates to the readme
- Be more assertive
- Another one bites the dust
- Fix more broken links
- Fix broken links
- Bump version one more time
- Fix readme
- Update docs to reflect changes in API
- Update README.md
- simplify ci requirements
- Update README.md
- Update README.md
- Update README.md
- Update docs a little
- move fmting to trait and add github actions support
- Rename more of the API
- Rename context apis
- Rename Context trait
- Rename core types
- Document no-std support
- Use instrs.json as file path in examples
- Capitalize example error messages
- Add brief comparison with thiserror
- Remove fehler link
- Pull in thiserror library for tests
- Link to thiserror instead of err-derive
- Rephrase relationship to fehler after lots of changes
- Raise minimum supported rustc version to 1.34
- Show supported compiler version in readme
- Add comparison to failure crate
- Release 1.0.0
- Omit '0:' if only one cause
- Capitalize lines of Debug representation
- Include example of anyhow! macro
- No longer identical
- Link to err-derive crate
- Include more information in readme
- Add one sentence project summary
- Add acknowledgement of fehler::Exception
- Add readme stub

## [0.3.1] - 2021-06-24
# Fixed
- Fixed lifetime inference error caused by recent `std` change.


<!-- next-url -->
[Unreleased]: https://github.com/eyre-rs/simple-eyre/compare/v0.3.1...HEAD
[0.3.1]: https://github.com/eyre-rs/simple-eyre/releases/tag/v0.3.1
