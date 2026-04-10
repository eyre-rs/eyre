# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## [Unreleased] - ReleaseDate

## [1.0.0](https://github.com/eyre-rs/eyre/compare/eyre-v0.6.12...eyre-v1.0.0) - 2026-04-10

### Added

- introduce an `"anyhow"` compatibility layer feature flag ([#138](https://github.com/eyre-rs/eyre/pull/138))

### Fixed

- typos in documentation files ([#210](https://github.com/eyre-rs/eyre/pull/210))
- consistent naming and duplicated tests
- import consistency
- typo in doc comment
- *(docs)* enclose inline code in backticks ([#170](https://github.com/eyre-rs/eyre/pull/170))
- remove `anyhow` feature flag from `OptionExt` location test  ([#148](https://github.com/eyre-rs/eyre/pull/148))
- *(docs)* fix two `rustdoc::bare_urls` warnings ([#139](https://github.com/eyre-rs/eyre/pull/139))
- `ok_or_eyre` not using `track_caller` ([#140](https://github.com/eyre-rs/eyre/pull/140))
- fixed minor errors in README, ?enhanced? punny-ness ([#33](https://github.com/eyre-rs/eyre/pull/33))
- fix type inference issue in eyre macro autoderef behavior ([#27](https://github.com/eyre-rs/eyre/pull/27))

### Other

- Bump edition to 2024 ([#279](https://github.com/eyre-rs/eyre/pull/279))
- Add default type parameter value of `T = ()` to `eyre::Result<T>` ([#273](https://github.com/eyre-rs/eyre/pull/273))
- Remove pyo3 feature ([#278](https://github.com/eyre-rs/eyre/pull/278))
- Rename the `WrapErr` trait to `ResultExt` ([#270](https://github.com/eyre-rs/eyre/pull/270))
- switch from `once_cell` to `std::sync::OnceLock` ([#218](https://github.com/eyre-rs/eyre/pull/218))
- Update the MSRV to 1.85 ([#276](https://github.com/eyre-rs/eyre/pull/276))
- Remove unnecessary `cfg` due to MSRV increase ([#275](https://github.com/eyre-rs/eyre/pull/275))
- Remove `format_err!` macro ([#274](https://github.com/eyre-rs/eyre/pull/274))
- [**breaking**] remove alias exports `DefaultContext` and `EyreContext` ([#181](https://github.com/eyre-rs/eyre/pull/181))
- [**breaking**] remove `anyhow` from default features ([#180](https://github.com/eyre-rs/eyre/pull/180))
- update broken link ([#220](https://github.com/eyre-rs/eyre/pull/220))
- Fix broken links
- Make cargo.toml consistent
- Add simple-eyre to workspace
- Use doc_auto_cfg for docs.rs
- Use workspace authors info
- Remove doc(html_root_url)
- Bump pyo3 dependency to fix the CI error
- Exclude development scripts from published package
- Add release-plz to CI ([#242](https://github.com/eyre-rs/eyre/pull/242))
- Redo eyre version bump
- Undo eyre version bump (DO NOT PUBLISH EYRE)
- re-bump eyre version
- Undo eyre version bump
- Merge branch 'clippy' into pyo3
- Update pyo3
- emit rustc-check-cfg info and fix doc_lazy_continuation clippy warning ([#200](https://github.com/eyre-rs/eyre/pull/200))
- Merge branch 'master' into fix-nightly-backtraces
- Merge branch 'master' into fix-ambiguous-methods
- Remove empty doc comments
- Merge branch 'master' into bump-owo-colors-for-everyone
- Don't evaluate 1-argument `ensure!` condition twice ([#166](https://github.com/eyre-rs/eyre/pull/166))
- Fix ci breaking due to new dependencies and rustc changes (backport to master) ([#163](https://github.com/eyre-rs/eyre/pull/163))
- Merge remote-tracking branch 'origin/master' into color-eyre
- Bump version 1.0.0 ([#146](https://github.com/eyre-rs/eyre/pull/146))
- Release 0.6.11 ([#134](https://github.com/eyre-rs/eyre/pull/134))
- Fix invalid drop impl call in `Report::downcast` ([#143](https://github.com/eyre-rs/eyre/pull/143))
- Revert "Automatically convert to external errors w/ ensure! and bail!" ([#133](https://github.com/eyre-rs/eyre/pull/133))
- Release 0.6.10 ([#132](https://github.com/eyre-rs/eyre/pull/132))
- Extend `Option` with `ok_or_eyre` ([#129](https://github.com/eyre-rs/eyre/pull/129))
- Add `eyre::Ok` ([#91](https://github.com/eyre-rs/eyre/pull/91))
- Automatically convert to external errors w/ ensure! and bail! ([#95](https://github.com/eyre-rs/eyre/pull/95))
- Add documentation on `wrap_err` vs `wrap_err_with` ([#93](https://github.com/eyre-rs/eyre/pull/93))
- fix references to `Error` in docstrings
- Add 1-argument `ensure!($expr)`
- remove deprecated lints as of 1.74
- Release independent packages
- manually apply cargo-release substitutions and also prep color-spantrace for publish
- Update changelog for 0.6.9 release
- Remove obsolete private_in_public lint in nightly and beta. ([#113](https://github.com/eyre-rs/eyre/pull/113))
- Update future-incompat pyo3 dependency
- Avoid filename collision in the monorepo structure
- Update documentation on no_std support. ([#111](https://github.com/eyre-rs/eyre/pull/111))
- Add build script to check for nightly ([#112](https://github.com/eyre-rs/eyre/pull/112))
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
### Added
- feature flag for `anyhow` compatibility traits [by LeoniePhiline](https://github.com/eyre-rs/eyre/pull/138)

## [0.6.11] - 2023-12-13
### Fixed
- stale references to `Error` in docstrings [by birkenfeld](https://github.com/eyre-rs/eyre/pull/87)

### Added
- one-argument ensure!($expr) [by sharnoff](https://github.com/eyre-rs/eyre/pull/86)
- documentation on the performance characteristics of `wrap_err` vs `wrap_err_with` [by akshayknarayan](https://github.com/eyre-rs/eyre/pull/93)
    - tl;dr: `wrap_err_with` is faster unless the constructed error object already exists
- ~~automated conversion to external errors for ensure! and bail! [by j-baker](https://github.com/eyre-rs/eyre/pull/95)~~ breaking change: shelved for next major release
- eyre::Ok for generating eyre::Ok() without fully specifying the type [by kylewlacy](https://github.com/eyre-rs/eyre/pull/91)
- `OptionExt::ok_or_eyre` for yielding static `Report`s from `None` [by LeoniePhiline](https://github.com/eyre-rs/eyre/pull/125)

### New Contributors
- @sharnoff made their first contribution in https://github.com/eyre-rs/eyre/pull/86
- @akshayknarayan made their first contribution in https://github.com/eyre-rs/eyre/pull/93
- @j-baker made their first contribution in https://github.com/eyre-rs/eyre/pull/95
- @kylewlacy made their first contribution in https://github.com/eyre-rs/eyre/pull/91
- @LeoniePhiline made their first contribution in https://github.com/eyre-rs/eyre/pull/129

~~## [0.6.10] - 2023-12-07~~ Yanked

## [0.6.9] - 2023-11-17
### Fixed
- stacked borrows when dropping [by TimDiekmann](https://github.com/eyre-rs/eyre/pull/81)
- miri validation errors through now stricter provenance [by ten3roberts](https://github.com/eyre-rs/eyre/pull/103)
- documentation on no_std support [by thenorili](https://github.com/eyre-rs/eyre/pull/111)

### Added
- monorepo for eyre-related crates [by pksunkara](https://github.com/eyre-rs/eyre/pull/104), [[2]](https://github.com/eyre-rs/eyre/pull/105)[[3]](https://github.com/eyre-rs/eyre/pull/107)
- CONTRIBUTING.md [by yaahc](https://github.com/eyre-rs/eyre/pull/99)

## [0.6.8] - 2022-04-04
### Added
- `#[must_use]` to `Report`
- `must-install` feature to help reduce binary sizes when using a custom `EyreHandler`

## [0.6.7] - 2022-02-24
### Fixed
- missing track_caller annotation to new format arg capture constructor

## [0.6.6] - 2022-01-19
### Added
- support for format arguments capture on 1.58 and later

## [0.6.5] - 2021-01-05
### Added
- optional support for converting into `pyo3` exceptions

## [0.6.4] - 2021-01-04
### Fixed
- missing track_caller annotations to `wrap_err` related trait methods

## [0.6.3] - 2020-11-10
### Fixed
- missing track_caller annotation to autoref specialization functions

## [0.6.2] - 2020-10-27
### Fixed
- missing track_caller annotation to new_adhoc function

## [0.6.1] - 2020-09-28
### Added
- support for track_caller on rust versions where it is available


<!-- next-url -->
[Unreleased]: https://github.com/eyre-rs/eyre/compare/v0.6.11...HEAD
[0.6.11]: https://github.com/eyre-rs/eyre/compare/v0.6.9...v0.6.11
[0.6.9]:  https://github.com/eyre-rs/eyre/compare/v0.6.8...v0.6.9
[0.6.8]:  https://github.com/eyre-rs/eyre/compare/v0.6.7...v0.6.8
[0.6.7]:  https://github.com/eyre-rs/eyre/compare/v0.6.6...v0.6.7
[0.6.6]:  https://github.com/eyre-rs/eyre/compare/v0.6.5...v0.6.6
[0.6.5]:  https://github.com/eyre-rs/eyre/compare/v0.6.4...v0.6.5
[0.6.4]:  https://github.com/eyre-rs/eyre/compare/v0.6.3...v0.6.4
[0.6.3]:  https://github.com/eyre-rs/eyre/compare/v0.6.2...v0.6.3
[0.6.2]:  https://github.com/eyre-rs/eyre/compare/v0.6.1...v0.6.2
[0.6.1]:  https://github.com/eyre-rs/eyre/releases/tag/v0.6.1
