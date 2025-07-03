# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Added

- This buildpack now prunes npm dev dependencies at the end of its build to reduce the final image size. ([#1126](https://github.com/heroku/buildpacks-nodejs/pull/1126))

## [3.6.8] - 2025-06-19

- No changes.

## [3.6.7] - 2025-06-12

- No changes.

## [3.6.6] - 2025-05-22

- No changes.

## [3.6.5] - 2025-05-16

- No changes.

## [3.6.4] - 2025-05-09

- No changes.

## [3.6.3] - 2025-05-07

- No changes.

## [3.6.2] - 2025-05-05

### Changed

- Buildpack output changes. ([#1093](https://github.com/heroku/buildpacks-nodejs/pull/1093))

## [3.6.1] - 2025-04-24

### Changed

- Updated error messages and formatting. ([#1076](https://github.com/heroku/buildpacks-nodejs/pull/1076))

## [3.6.0] - 2025-04-09

- No changes.

## [3.5.3] - 2025-04-03

### Changed

- Updated libcnb to 0.28.1, which includes tracing improvements/fixes. ([#1063](https://github.com/heroku/buildpacks-nodejs/pull/1063))

## [3.5.2] - 2025-03-28

- No changes.

## [3.5.1] - 2025-03-14

- No changes.

## [3.5.0] - 2025-02-28

### Changed

- Buildpack output is sent to `stderr` instead of `stdout`. ([#1039](https://github.com/heroku/buildpacks-nodejs/pull/1039))

## [3.4.11] - 2025-02-27

- No changes.

## [3.4.10] - 2025-02-25

- No changes.

## [3.4.9] - 2025-02-21

- No changes.

## [3.4.8] - 2025-02-14

- No changes.

## [3.4.7] - 2025-02-12

- No changes.

## [3.4.6] - 2025-02-11

- No changes.

## [3.4.5] - 2025-02-03

- No changes.

## [3.4.4] - 2025-01-22

- No changes.

## [3.4.3] - 2025-01-10

### Removed

- Check and warning for `node_modules` directory is unnecessary since
  `npm ci` will remove it before installing dependencies. ([#998]https://github.com/heroku/buildpacks-nodejs/pull/998))

## [3.4.2] - 2025-01-08

- No changes.

## [3.4.1] - 2025-01-07

### Changed

- Replaced `commons` output module with `bullet_stream` ([#993](https://github.com/heroku/buildpacks-nodejs/pull/993))

## [3.4.0] - 2024-12-13

### Changed

- Default processes will no longer be registered if a Procfile is present. ([#985](https://github.com/heroku/buildpacks-nodejs/pull/985))

## [3.3.5] - 2024-12-11

- No changes.

## [3.3.4] - 2024-12-05

- No changes.

## [3.3.3] - 2024-11-22

- No changes.

## [3.3.2] - 2024-11-13

- No changes.

## [3.3.1] - 2024-11-06

- No changes.

## [3.3.0] - 2024-10-31

- No changes.

## [3.2.18] - 2024-10-31

- No changes.

## [3.2.17] - 2024-10-25

- No changes.

## [3.2.16] - 2024-10-22

### Added

- Allow configuration of build script behavior through the `node_build_scripts` build plan. ([#928](https://github.com/heroku/buildpacks-nodejs/pull/928))

## [3.2.15] - 2024-10-04

- No changes.

## [3.2.14] - 2024-09-24

- No changes.

## [3.2.13] - 2024-09-04

- No changes.

## [3.2.12] - 2024-08-27

- No changes.

## [3.2.11] - 2024-08-12

- No changes.

## [3.2.10] - 2024-07-29

- No changes.

## [3.2.9] - 2024-07-19

- No changes.

## [3.2.8] - 2024-07-18

- No changes.

## [3.2.7] - 2024-07-09

- No changes.

## [3.2.6] - 2024-07-03

- No changes.

## [3.2.5] - 2024-06-21

- No changes.

## [3.2.4] - 2024-06-13

- No changes.

## [3.2.3] - 2024-05-29

- Fix for npm permission error when executing without root
  permissions ([#830](https://github.com/heroku/buildpacks-nodejs/pull/830))

## [3.2.2] - 2024-05-22

- No changes.

## [3.2.1] - 2024-05-10

- No changes.

## [3.2.0] - 2024-05-09

- No changes.

## [3.1.0] - 2024-05-09

### Added

- Support for `arm64` and multi-arch images. ([#815](https://github.com/heroku/buildpacks-nodejs/pull/815))

## [3.0.6] - 2024-05-03

- No changes.

## [3.0.5] - 2024-04-25

- No changes.

## [3.0.4] - 2024-04-10

- No changes.

## [3.0.3] - 2024-04-04

- No changes.

## [3.0.2] - 2024-03-27

- No changes.

## [3.0.1] - 2024-03-11

- No changes.

## [3.0.0] - 2024-03-08

- Bump to Buildpack API 0.10.
  ([#789](https://github.com/heroku/buildpacks-nodejs/pull/789))

## [2.6.6] - 2024-02-15

- No changes.

## [2.6.5] - 2024-02-01

- No changes.

## [2.6.4] - 2024-01-17

- No changes.

## [2.6.3] - 2024-01-11

- No changes.

## [2.6.2] - 2024-01-02

- No changes.

## [2.6.1] - 2023-12-14

- No changes.

## [2.6.0] - 2023-12-14

- No changes.

## [2.5.0] - 2023-12-07

### Added

- Enabled libcnb `trace` feature, so that OpenTelemetry file exports with
  buildpack detect and build traces are emitted to the file system.
  ([#749](https://github.com/heroku/buildpacks-nodejs/pull/749))

## [2.4.1] - 2023-12-04

- No changes.

## [2.4.0] - 2023-12-01

### Fixed

- Detection will report a failure instead of throwing an error when there is no `package.json` file in the application directory. ([#733](https://github.com/heroku/buildpacks-nodejs/pull/733))

## [2.3.0] - 2023-11-09

- No changes.

## [2.2.0] - 2023-10-26

- No changes.

## [2.1.0] - 2023-10-26

- No changes.

## [2.0.0] - 2023-10-24

### Added

- Initial release

[unreleased]: https://github.com/heroku/buildpacks-nodejs/compare/v3.6.8...HEAD
[3.6.8]: https://github.com/heroku/buildpacks-nodejs/compare/v3.6.7...v3.6.8
[3.6.7]: https://github.com/heroku/buildpacks-nodejs/compare/v3.6.6...v3.6.7
[3.6.6]: https://github.com/heroku/buildpacks-nodejs/compare/v3.6.5...v3.6.6
[3.6.5]: https://github.com/heroku/buildpacks-nodejs/compare/v3.6.4...v3.6.5
[3.6.4]: https://github.com/heroku/buildpacks-nodejs/compare/v3.6.3...v3.6.4
[3.6.3]: https://github.com/heroku/buildpacks-nodejs/compare/v3.6.2...v3.6.3
[3.6.2]: https://github.com/heroku/buildpacks-nodejs/compare/v3.6.1...v3.6.2
[3.6.1]: https://github.com/heroku/buildpacks-nodejs/compare/v3.6.0...v3.6.1
[3.6.0]: https://github.com/heroku/buildpacks-nodejs/compare/v3.5.3...v3.6.0
[3.5.3]: https://github.com/heroku/buildpacks-nodejs/compare/v3.5.2...v3.5.3
[3.5.2]: https://github.com/heroku/buildpacks-nodejs/compare/v3.5.1...v3.5.2
[3.5.1]: https://github.com/heroku/buildpacks-nodejs/compare/v3.5.0...v3.5.1
[3.5.0]: https://github.com/heroku/buildpacks-nodejs/compare/v3.4.11...v3.5.0
[3.4.11]: https://github.com/heroku/buildpacks-nodejs/compare/v3.4.10...v3.4.11
[3.4.10]: https://github.com/heroku/buildpacks-nodejs/compare/v3.4.9...v3.4.10
[3.4.9]: https://github.com/heroku/buildpacks-nodejs/compare/v3.4.8...v3.4.9
[3.4.8]: https://github.com/heroku/buildpacks-nodejs/compare/v3.4.7...v3.4.8
[3.4.7]: https://github.com/heroku/buildpacks-nodejs/compare/v3.4.6...v3.4.7
[3.4.6]: https://github.com/heroku/buildpacks-nodejs/compare/v3.4.5...v3.4.6
[3.4.5]: https://github.com/heroku/buildpacks-nodejs/compare/v3.4.4...v3.4.5
[3.4.4]: https://github.com/heroku/buildpacks-nodejs/compare/v3.4.3...v3.4.4
[3.4.3]: https://github.com/heroku/buildpacks-nodejs/compare/v3.4.2...v3.4.3
[3.4.2]: https://github.com/heroku/buildpacks-nodejs/compare/v3.4.1...v3.4.2
[3.4.1]: https://github.com/heroku/buildpacks-nodejs/compare/v3.4.0...v3.4.1
[3.4.0]: https://github.com/heroku/buildpacks-nodejs/compare/v3.3.5...v3.4.0
[3.3.5]: https://github.com/heroku/buildpacks-nodejs/compare/v3.3.4...v3.3.5
[3.3.4]: https://github.com/heroku/buildpacks-nodejs/compare/v3.3.3...v3.3.4
[3.3.3]: https://github.com/heroku/buildpacks-nodejs/compare/v3.3.2...v3.3.3
[3.3.2]: https://github.com/heroku/buildpacks-nodejs/compare/v3.3.1...v3.3.2
[3.3.1]: https://github.com/heroku/buildpacks-nodejs/compare/v3.3.0...v3.3.1
[3.3.0]: https://github.com/heroku/buildpacks-nodejs/compare/v3.2.18...v3.3.0
[3.2.18]: https://github.com/heroku/buildpacks-nodejs/compare/v3.2.17...v3.2.18
[3.2.17]: https://github.com/heroku/buildpacks-nodejs/compare/v3.2.16...v3.2.17
[3.2.16]: https://github.com/heroku/buildpacks-nodejs/compare/v3.2.15...v3.2.16
[3.2.15]: https://github.com/heroku/buildpacks-nodejs/compare/v3.2.14...v3.2.15
[3.2.14]: https://github.com/heroku/buildpacks-nodejs/compare/v3.2.13...v3.2.14
[3.2.13]: https://github.com/heroku/buildpacks-nodejs/compare/v3.2.12...v3.2.13
[3.2.12]: https://github.com/heroku/buildpacks-nodejs/compare/v3.2.11...v3.2.12
[3.2.11]: https://github.com/heroku/buildpacks-nodejs/compare/v3.2.10...v3.2.11
[3.2.10]: https://github.com/heroku/buildpacks-nodejs/compare/v3.2.9...v3.2.10
[3.2.9]: https://github.com/heroku/buildpacks-nodejs/compare/v3.2.8...v3.2.9
[3.2.8]: https://github.com/heroku/buildpacks-nodejs/compare/v3.2.7...v3.2.8
[3.2.7]: https://github.com/heroku/buildpacks-nodejs/compare/v3.2.6...v3.2.7
[3.2.6]: https://github.com/heroku/buildpacks-nodejs/compare/v3.2.5...v3.2.6
[3.2.5]: https://github.com/heroku/buildpacks-nodejs/compare/v3.2.4...v3.2.5
[3.2.4]: https://github.com/heroku/buildpacks-nodejs/compare/v3.2.3...v3.2.4
[3.2.3]: https://github.com/heroku/buildpacks-nodejs/compare/v3.2.2...v3.2.3
[3.2.2]: https://github.com/heroku/buildpacks-nodejs/compare/v3.2.1...v3.2.2
[3.2.1]: https://github.com/heroku/buildpacks-nodejs/compare/v3.2.0...v3.2.1
[3.2.0]: https://github.com/heroku/buildpacks-nodejs/compare/v3.1.0...v3.2.0
[3.1.0]: https://github.com/heroku/buildpacks-nodejs/compare/v3.0.6...v3.1.0
[3.0.6]: https://github.com/heroku/buildpacks-nodejs/compare/v3.0.5...v3.0.6
[3.0.5]: https://github.com/heroku/buildpacks-nodejs/compare/v3.0.4...v3.0.5
[3.0.4]: https://github.com/heroku/buildpacks-nodejs/compare/v3.0.3...v3.0.4
[3.0.3]: https://github.com/heroku/buildpacks-nodejs/compare/v3.0.2...v3.0.3
[3.0.2]: https://github.com/heroku/buildpacks-nodejs/compare/v3.0.1...v3.0.2
[3.0.1]: https://github.com/heroku/buildpacks-nodejs/compare/v3.0.0...v3.0.1
[3.0.0]: https://github.com/heroku/buildpacks-nodejs/compare/v2.6.6...v3.0.0
[2.6.6]: https://github.com/heroku/buildpacks-nodejs/compare/v2.6.5...v2.6.6
[2.6.5]: https://github.com/heroku/buildpacks-nodejs/compare/v2.6.4...v2.6.5
[2.6.4]: https://github.com/heroku/buildpacks-nodejs/compare/v2.6.3...v2.6.4
[2.6.3]: https://github.com/heroku/buildpacks-nodejs/compare/v2.6.2...v2.6.3
[2.6.2]: https://github.com/heroku/buildpacks-nodejs/compare/v2.6.1...v2.6.2
[2.6.1]: https://github.com/heroku/buildpacks-nodejs/compare/v2.6.0...v2.6.1
[2.6.0]: https://github.com/heroku/buildpacks-nodejs/compare/v2.5.0...v2.6.0
[2.5.0]: https://github.com/heroku/buildpacks-nodejs/compare/v2.4.1...v2.5.0
[2.4.1]: https://github.com/heroku/buildpacks-nodejs/compare/v2.4.0...v2.4.1
[2.4.0]: https://github.com/heroku/buildpacks-nodejs/compare/v2.3.0...v2.4.0
[2.3.0]: https://github.com/heroku/buildpacks-nodejs/compare/v2.2.0...v2.3.0
[2.2.0]: https://github.com/heroku/buildpacks-nodejs/compare/v2.1.0...v2.2.0
[2.1.0]: https://github.com/heroku/buildpacks-nodejs/compare/v2.0.0...v2.1.0
[2.0.0]: https://github.com/heroku/buildpacks-nodejs/releases/tag/v2.0.0
