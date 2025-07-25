# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [4.1.0] - 2025-07-22

### Added

- [experimental] Expose dev dependency pruning as a project.toml config. ([#1146](https://github.com/heroku/buildpacks-nodejs/pull/1146))

## [4.0.1] - 2025-07-17

- No changes.

## [4.0.0] - 2025-07-10

### Added

- This buildpack now prunes Yarn dev dependencies at the end of its build to reduce the final image size. ([#1136](https://github.com/heroku/buildpacks-nodejs/pull/1136))

### Changed

- The Yarn dependency cache will only be included in the final image when Plug'n'Play mode is enabled. ([#1136](https://github.com/heroku/buildpacks-nodejs/pull/1136))

## [3.6.11] - 2025-07-09

- No changes.

## [3.6.10] - 2025-06-25

- No changes.

## [3.6.9] - 2025-06-23

- No changes.

## [3.6.8] - 2025-06-19

### Changed

- Replace Yarn version resolution from inventory to packument-based resolver. ([#1122](https://github.com/heroku/buildpacks-nodejs/pull/1122))

## [3.6.7] - 2025-06-12

- Added Yarn version 4.9.2.

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
- Added retry behavior for Yarn binary downloads ([#1092](https://github.com/heroku/buildpacks-nodejs/pull/1092))

## [3.6.1] - 2025-04-24

### Added 

- Added Yarn version 4.9.1.
- Added Yarn version 4.9.0.

### Changed

- Updated error messages and formatting. ([#1074](https://github.com/heroku/buildpacks-nodejs/pull/1074))

## [3.6.0] - 2025-04-09

- No changes.

## [3.5.3] - 2025-04-03

### Changed

- Updated libcnb to 0.28.1, which includes tracing improvements/fixes. ([#1063](https://github.com/heroku/buildpacks-nodejs/pull/1063))

### Added 

- Added Yarn version 4.8.1.
- Added Yarn version 4.8.0.

## [3.5.2] - 2025-03-28

- No changes.

## [3.5.1] - 2025-03-14

- Added Yarn version 4.7.0.

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

- No changes.

## [3.4.2] - 2025-01-08

- No changes.

## [3.4.1] - 2025-01-07

- Added Yarn version 4.6.0.

## [3.4.0] - 2024-12-13

### Changed

- Default processes will no longer be registered if a Procfile is present. ([#985](https://github.com/heroku/buildpacks-nodejs/pull/985))

## [3.3.5] - 2024-12-11

- No changes.

## [3.3.4] - 2024-12-05

- Added Yarn version 4.5.3.
- Added Yarn version 3.8.7.

## [3.3.3] - 2024-11-22

- Added Yarn version 4.5.2.

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
- Added Yarn version 4.5.1.
- Added Yarn version 3.8.6.

## [3.2.15] - 2024-10-04

- No changes.

## [3.2.14] - 2024-09-24

- Added Yarn version 4.5.0.

## [3.2.13] - 2024-09-04

- No changes.

## [3.2.12] - 2024-08-27

- Added Yarn version 4.4.1.
- Added Yarn version 3.8.5.

## [3.2.11] - 2024-08-12

- Added Yarn version 4.4.0.
- Added Yarn version 3.8.4.

## [3.2.10] - 2024-07-29

- No changes.

## [3.2.9] - 2024-07-19

- No changes.

## [3.2.8] - 2024-07-18

- No changes.

## [3.2.7] - 2024-07-09

- No changes.

## [3.2.6] - 2024-07-03

- Added Yarn version 4.3.1.
- Added Yarn version 3.8.3.

## [3.2.5] - 2024-06-21

- No changes.

## [3.2.4] - 2024-06-13

- Added Yarn version 4.3.0.

## [3.2.3] - 2024-05-29

- No changes.

## [3.2.2] - 2024-05-22

- No changes.

## [3.2.1] - 2024-05-10

- Added Yarn version 4.2.2.

## [3.2.0] - 2024-05-09

- No changes.

## [3.1.0] - 2024-05-09

### Added

- Support for `arm64` and multi-arch images. ([#815](https://github.com/heroku/buildpacks-nodejs/pull/815))

## [3.0.6] - 2024-05-03

- Added Yarn version 4.2.1.
- Added Yarn version 4.2.0.
- Added Yarn version 3.8.2.

## [3.0.5] - 2024-04-25

- No changes.

## [3.0.4] - 2024-04-10

- No changes.

## [3.0.3] - 2024-04-04

- No changes.

## [3.0.2] - 2024-03-27

- No changes.

## [3.0.1] - 2024-03-11

- Added Yarn version 1.22.22.

## [3.0.0] - 2024-03-08

- Bump to Buildpack API 0.10.
  ([#789](https://github.com/heroku/buildpacks-nodejs/pull/789))
- Added Yarn version 4.1.1.
- Added Yarn version 3.8.1.

## [2.6.6] - 2024-02-15

- Added Yarn version 3.8.0.

## [2.6.5] - 2024-02-01

- Added Yarn version 4.1.0.

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

### Added

- Added Yarn version 4.0.2.
- Added Yarn version 3.7.0.
- Added Yarn version 1.22.21.
- Added Yarn version 1.22.20.

## [2.3.0] - 2023-11-09

- Added Yarn version 4.0.1.

## [2.2.0] - 2023-10-26

- No changes.

## [2.1.0] - 2023-10-26

### Added

- Support for Yarn 4. ([#698](https://github.com/heroku/buildpacks-nodejs/pull/698)
- Added Yarn version 4.0.0. ([#702](https://github.com/heroku/buildpacks-nodejs/pull/702))

### Changed

- Now sets `enableGlobalCache` to `false` for Yarn 2+ builds. ([#698](https://github.com/heroku/buildpacks-nodejs/pull/698))

## [2.0.0] - 2023-10-24

### Changed

- Updated buildpack description and keywords. ([#692](https://github.com/heroku/buildpacks-nodejs/pull/692))

### Removed

- Removed redundant explicitly named supported stacks. ([#693](https://github.com/heroku/buildpacks-nodejs/pull/693))

## [1.1.7] - 2023-10-17

- Added Yarn version 4.0.0-rc.53.
- Added Yarn version 4.0.0-rc.52.
- Added Yarn version 3.6.4.

## [1.1.6] - 2023-09-25

- No changes.

## [1.1.5] - 2023-09-19

- Added Yarn version 4.0.0-rc.51.
- Added Yarn version 4.0.0-rc.50.
- Added Yarn version 4.0.0-rc.49.
- Added Yarn version 3.6.3.
- Added Yarn version 3.6.2.

## [1.1.4] - 2023-08-10

- No changes.

## [1.1.3] - 2023-07-24

- No changes.

## [1.1.2] - 2023-07-19

- No changes

## [1.1.1] - 2023-07-07

- Added Yarn version 3.6.1, 4.0.0-rc-47, 4.0.0-rc.48.

## [1.1.0] - 2023-06-28

- Added Yarn version 4.0.0-rc.46.

## [0.4.4] - 2023-06-14

- Added Yarn version 3.6.0.
- Upgrade to Buildpack API version `0.9`. ([#552](https://github.com/heroku/buildpacks-nodejs/pull/552))

## [0.4.3] - 2023-05-22

- Change release target from ECR to docker.io/heroku/buildpack-nodejs-yarn.
- Drop explicit support for the End-of-Life stack `heroku-18`.
- Added yarn version 4.0.0-rc.44.

## [0.4.2] - 2023-05-08

- Added yarn version 3.5.1, 4.0.0-rc.43.

## [0.4.1] - 2023-04-03

- Added yarn version 4.0.0-rc.42.
- Added yarn version 4.0.0-rc.41.
- Added yarn version 3.5.0.
- Added yarn version 4.0.0-rc.40.

## [0.4.0] - 2023-02-27

- Add several yarn 2, 3, and 4 releases to inventory ([#457](https://github.com/heroku/buildpacks-nodejs/pull/457))

## [0.3.2] - 2023-02-02

- `name` is no longer a required field in package.json. ([#447](https://github.com/heroku/buildpacks-nodejs/pull/447))

## [0.3.1] - 2023-01-17

- No longer installs `yarn` if it's already been installed by another buildpack,
  like heroku/nodejs-corepack ([#418](https://github.com/heroku/buildpacks-nodejs/pull/418))

## [0.3.0] - 2022-12-05

- Rewrite in rust leveraging libcnb.rs ([#250](https://github.com/heroku/buildpacks-nodejs/pull/250/files))
- Update to buildpack API version 0.8 ([#250](https://github.com/heroku/buildpacks-nodejs/pull/250/files))
- Added explicit support for yarn 2 and 3 ([#250](https://github.com/heroku/buildpacks-nodejs/pull/250/files))
- Added support for yarn zero-installs and pnp ([#250](https://github.com/heroku/buildpacks-nodejs/pull/250/files))
- No longer installs or relies on yj ([#250](https://github.com/heroku/buildpacks-nodejs/pull/250/files))
- No longer caches or restores node_modules folder ([#250](https://github.com/heroku/buildpacks-nodejs/pull/250/files))

## [0.2.3] - 2022-04-05

- Add support for the heroku-22 stack

## [0.2.2] - 2022-04-04

- `yarn install` now run with `--production=false` to ensure `devDependencies` are installed ([201](https://github.com/heroku/buildpacks-nodejs/pull/201))

## [0.2.1] - 2022-03-23

- The `web` process affiliated with `package.json`'s `scripts.start` is now a `default` process ([#214](https://github.com/heroku/buildpacks-nodejs/pull/214))

## [0.2.0] - 2022-03-09

- Installs `yq` in the build toolbox layer ([#184](https://github.com/heroku/buildpacks-nodejs/pull/184))

## [0.1.8] - 2021-11-10

- install yarn
- upgrade to buildpack api 0.6
- support '*' stack

## [0.1.6] - 2021-08-04

### Fixed
- yarn buildpack consumes dependency on node during plan resolution
- cover yarn/npm buildpacks logic with tests

## [0.1.5] - 2021-06-17

### Fixed
- Empty cache builds no longer fail with a `PREV_NODE_VERSION ` unbound variable error ([#86](https://github.com/heroku/buildpacks-node/pull/86))

## [0.1.4] - 2021-06-15

### Fixed
- Clear cache when node version changes ([#40](https://github.com/heroku/buildpacks-node/pull/40))

## [0.1.3] - 2021-03-04

- Add license to buildpack.toml ([#17](https://github.com/heroku/buildpacks-node/pull/17))
- Flush cache when stack image changes ([#28](https://github.com/heroku/buildpacks-node/pull/28))
- Trim whitespace when getting stack name ([#29](https://github.com/heroku/buildpacks-node/pull/29))
- Fail if two lock files are detected ([#30](https://github.com/heroku/buildpacks-node/pull/30))

## [0.1.1] - 2021-01-20

## [0.1.0] - 2020-11-11

### Added
- Add support for heroku-20 and bionic stacks ([#4](https://github.com/heroku/nodejs-yarn-buildpack/pull/4))

## [0.0.1] - 2019-12-08

### Added
- Changelog entry for first release ([#1](https://github.com/heroku/nodejs-yarn-buildpack/pull/1))

[unreleased]: https://github.com/heroku/buildpacks-nodejs/compare/v4.1.0...HEAD
[4.1.0]: https://github.com/heroku/buildpacks-nodejs/compare/v4.0.1...v4.1.0
[4.0.1]: https://github.com/heroku/buildpacks-nodejs/compare/v4.0.0...v4.0.1
[4.0.0]: https://github.com/heroku/buildpacks-nodejs/compare/v3.6.11...v4.0.0
[3.6.11]: https://github.com/heroku/buildpacks-nodejs/compare/v3.6.10...v3.6.11
[3.6.10]: https://github.com/heroku/buildpacks-nodejs/compare/v3.6.9...v3.6.10
[3.6.9]: https://github.com/heroku/buildpacks-nodejs/compare/v3.6.8...v3.6.9
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
[2.0.0]: https://github.com/heroku/buildpacks-nodejs/compare/v1.1.7...v2.0.0
[1.1.7]: https://github.com/heroku/buildpacks-nodejs/compare/v1.1.6...v1.1.7
[1.1.6]: https://github.com/heroku/buildpacks-nodejs/compare/v1.1.5...v1.1.6
[1.1.5]: https://github.com/heroku/buildpacks-nodejs/compare/v1.1.4...v1.1.5
[1.1.4]: https://github.com/heroku/buildpacks-nodejs/compare/v1.1.3...v1.1.4
[1.1.3]: https://github.com/heroku/buildpacks-nodejs/compare/v1.1.2...v1.1.3
[1.1.2]: https://github.com/heroku/buildpacks-nodejs/compare/v1.1.1...v1.1.2
[1.1.1]: https://github.com/heroku/buildpacks-nodejs/compare/v1.1.0...v1.1.1
[1.1.0]: https://github.com/heroku/buildpacks-nodejs/releases/tag/v1.1.0
