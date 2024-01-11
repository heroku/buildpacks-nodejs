# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

[unreleased]: https://github.com/heroku/buildpacks-nodejs/compare/v2.6.3...HEAD
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
