# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## [Unreleased] - ReleaseDate

## [0.5.5] - 2020-09-21
### Added
- add `issue_filter` method to `HookBuilder` for disabling issue generation
  based on the error encountered.

## [0.5.4] - 2020-09-17
### Added
- Add new "issue-url" feature for generating issue creation links in error
  reports pre-populated with information about the error

## [0.5.3] - 2020-09-14
### Added
- add `panic_section` method to `HookBuilder` for overriding the printer for
  the panic message at the start of panic reports

## [0.5.2] - 2020-08-31
### Added
- make it so all `Section` trait methods can be called on `Report` in
  addition to the already supported usage on `Result<T, E: Into<Report>>`
- panic_section to `HookBuilder` to add custom sections to panic reports
- display_env_section to `HookBuilder` to disable the output indicating what
  environment variables can be set to manipulate the error reports
### Changed
- switched from ansi_term to owo-colors for colorizing output, allowing for
  better compatibility with the Display trait

<!-- next-url -->
[Unreleased]: https://github.com/yaahc/color-eyre/compare/v0.5.5...HEAD
[0.5.5]: https://github.com/yaahc/color-eyre/compare/v0.5.4...v0.5.5
[0.5.4]: https://github.com/yaahc/color-eyre/compare/v0.5.3...v0.5.4
[0.5.3]: https://github.com/yaahc/color-eyre/compare/v0.5.2...v0.5.3
[0.5.2]: https://github.com/yaahc/color-eyre/releases/tag/v0.5.2
