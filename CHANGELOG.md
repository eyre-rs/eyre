# Changelog
All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

<!-- next-header -->

## [Unreleased] - ReleaseDate

## [0.6.8] - 2022-04-04
### Added
- Add `#[must_use]` to `Report`
- Add `must-install` feature to help reduce binary sizes when using a custom `EyreHandler`

## [0.6.7] - 2022-02-24
### Fixed
- added missing track_caller annotation to new format arg capture constructor

## [0.6.6] - 2022-01-19
### Added
- add support for format arguments capture on 1.58 and later

## [0.6.5] - 2021-01-05
### Added
- add optional support for converting into `pyo3` exceptions

## [0.6.4] - 2021-01-04
### Fixed
- added missing track_caller annotations to `wrap_err` related trait methods

## [0.6.3] - 2020-11-10
### Fixed
- added missing track_caller annotation to autoref specialization functions

## [0.6.2] - 2020-10-27
### Fixed
- added missing track_caller annotation to new_adhoc function

## [0.6.1] - 2020-09-28
### Added
- support track_caller on rust versions where it is available


<!-- next-url -->
[Unreleased]: https://github.com/yaahc/eyre/compare/v0.6.8...HEAD
[0.6.8]: https://github.com/yaahc/eyre/compare/v0.6.7...v0.6.8
[0.6.7]: https://github.com/yaahc/eyre/compare/v0.6.6...v0.6.7
[0.6.6]: https://github.com/yaahc/eyre/compare/v0.6.5...v0.6.6
[0.6.5]: https://github.com/yaahc/eyre/compare/v0.6.4...v0.6.5
[0.6.4]: https://github.com/yaahc/eyre/compare/v0.6.3...v0.6.4
[0.6.3]: https://github.com/yaahc/eyre/compare/v0.6.2...v0.6.3
[0.6.2]: https://github.com/yaahc/eyre/compare/v0.6.1...v0.6.2
[0.6.1]: https://github.com/yaahc/eyre/releases/tag/v0.6.1
