# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog] and this project adheres to
[Semantic Versioning].

[Keep a Changelog]: http://keepachangelog.com/en/1.0.0/
[Semantic Versioning]: http://semver.org/spec/v2.0.0.html

## [Unreleased]

### Removed
- Broken `"unique"` feature.

### Changed
- Updated `abort_on_panic` dependency version to `"2.0.0"` from
  `"1.0.0"`.

### Added
- Setting `doc(html_root_url)` for inter-crate docs linking.
- Testing on Rust 1.20.0 now, as oldest supported version.

## [0.6.3] - 2018-03-05

### Fixed
- Heading in docs.

## [0.6.2] - 2017-11-13

### Changed
- Upgraded to `libffi-sys` 0.6.0, which uses an upgraded bindgen.

## [0.6.0] - 2017-05-14

### Fixed
- Marked `Unique::new` as `unsafe`.

### Added
- Mentions dependencies in build instructions.

### Changed
- Constructors and factories that need sequences now take `IntoIterator` 
instead of `Iterator` or `FixedSizeIterator`.

## [0.5.3] - 2017-04-15

### Fixed
- `Closure[0-9]` and `ClosureMut[0-9]` now abort on panic rather than 
attempting to unwind past an FFI boundary. (Thanks, ngkz!)

## [0.5.2] - 2017-04-14

### Changed
- Now depends on `libffi-sys` 0.5.0.
