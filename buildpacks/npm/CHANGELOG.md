# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

- No changes.

## [3.2.2] - 2024-05-22

- No changes.

## [3.2.1] - 2024-05-10

- No changes.

## [3.2.0] - 2024-05-09

- No changes.

## [3.1.0] - 2024-05-09

- No changes.

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

- No changes.

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

- No changes.

## [2.4.1] - 2023-12-04

- No changes.

## [2.4.0] - 2023-12-01

### Changed

- This buildpack now implements Buildpack API 0.7 instead of 0.6. ([#721](https://github.com/heroku/buildpacks-nodejs/pull/721))

## [2.3.0] - 2023-11-09

- No changes.

## [2.2.0] - 2023-10-26

- No changes.

## [2.1.0] - 2023-10-26

- No changes.

## [2.0.0] - 2023-10-24

### Changed

- Updated buildpack display name, description and keywords. ([#692](https://github.com/heroku/buildpacks-nodejs/pull/692))

### Removed

- Removed redundant explicitly named supported stacks. ([#693](https://github.com/heroku/buildpacks-nodejs/pull/693))

## [1.1.7] - 2023-10-17

- No changes.

## [1.1.6] - 2023-09-25

- No changes.

## [1.1.5] - 2023-09-19

- No changes.

## [1.1.4] - 2023-08-10

- No changes.

## [1.1.3] - 2023-07-24

- No changes.

## [1.1.2] - 2023-07-19

- No changes

## [1.1.1] - 2023-07-07

- No changes

## [1.1.0] - 2023-06-28

- No changes

## [0.5.3] - 2023-05-22

- Change release target from ECR to docker.io/heroku/buildpack-nodejs-npm.
- Drop explicit support for the End-of-Life stack `heroku-18`.

## [0.5.2] - 2022-04-05

- Add support for all stacks
- Add explicit support for the heroku-22 stack

## [0.5.1] - 2022-03-23

- The `web` process affiliated with `package.json`'s `scripts.start` is now a `default` process ([#214](https://github.com/heroku/buildpacks-nodejs/pull/214))

## [0.5.0] - 2022-03-09

- Upgraded to buildpack api 0.6 ([#184](https://github.com/heroku/buildpacks-nodejs/pull/184))
- Installs `yq` in the build toolbox layer ([#184](https://github.com/heroku/buildpacks-nodejs/pull/184))

## [0.4.5] - 2022-01-28

- `npm ci` and `npm install` now run with `--production=false` to ensure `devDependencies` are installed ([186](https://github.com/heroku/buildpacks-nodejs/pull/186))

## [0.4.4] - 2021-06-15

### Fixed
- Clear cache when node version changes ([#40](https://github.com/heroku/buildpacks-node/pull/40))

## [0.4.3] - 2021-03-04

- Flush cache when stack image changes ([#28](https://github.com/heroku/buildpacks-node/pull/28))
- Trim whitespace when getting stack name ([#29](https://github.com/heroku/buildpacks-node/pull/29))
- Fail if two lock files are detected ([#30](https://github.com/heroku/buildpacks-node/pull/30))

## [0.4.2] - 2021-02-23

- Add license to buildpack.toml ([#17](https://github.com/heroku/buildpacks-node/pull/17))

## [0.4.1] - 2021-01-20

- Ensure prefix directory exists ([#42](https://github.com/heroku/nodejs-npm-buildpack/pull/44))
- Use new logging style ([#45](https://github.com/heroku/nodejs-npm-buildpack/pull/45))
- Change log colors to use ANSI codes ([#47](https://github.com/heroku/nodejs-npm-buildpack/pull/47))

## [0.4.0] - 2020-11-11

### Added
- Add heroku-20 to supported stacks ([#40](https://github.com/heroku/nodejs-npm-buildpack/pull/40))

## [0.3.0] - 2020-09-16

### Added
- Prune devdependencies ([#32](https://github.com/heroku/nodejs-npm-buildpack/pull/32))
- Opt out of pruning devdependencies if NODE_ENV is not production ([#33](https://github.com/heroku/nodejs-npm-buildpack/pull/33))
- Warn when node modules are checked into git ([#34](https://github.com/heroku/nodejs-npm-buildpack/pull/34))
- Add logging method for warnings ([#35](https://github.com/heroku/nodejs-npm-buildpack/pull/35))
### Fixed
- Move integration testing to CirleCI ([#37](https://github.com/heroku/nodejs-npm-buildpack/pull/37))

## [0.2.0] - 2020-05-19

### Added
- docs: add docs around `Permission denied` issues ([#28](https://github.com/heroku/nodejs-npm-buildpack/pull/28))
- Add dockerized unit tests ([#29](https://github.com/heroku/nodejs-npm-buildpack/pull/29))
- Added `provides` and `requires` of `node_modules` and `node` to buildplan. ([#18](https://github.com/heroku/nodejs-npm-buildpack/pull/18))

## [0.1.4] - 2020-02-19

### Added
- feat: install `npm` version specified in `package.json` ([#24](https://github.com/heroku/nodejs-npm-buildpack/pull/24))
- feat: exchange echo commands for `log_info` method ([#25](https://github.com/heroku/nodejs-npm-buildpack/pull/25))
### Fixed
- fix: use_npm_ci expression return value id ([#22](https://github.com/heroku/nodejs-npm-buildpack/pull/23))

## [0.1.3] - 2020-01-28

### Fixed
- fix: remove `-buildpack` from buildpack id ([#16](https://github.com/heroku/nodejs-npm-buildpack/pull/16))
- feat: support running on `io.buildpacks.stacks.bionic` stack ([#17](https://github.com/heroku/nodejs-npm-buildpack/pull/17))

## [0.1.2] - 2019-11-01

### Added
- feat: support build time environment variables ([#14](https://github.com/heroku/nodejs-npm-buildpack/pull/14))

## [0.1.1] - 2019-10-30

### Fixed
- Fix copying node_modules when a `package-lock.json` is present ([#12](https://github.com/heroku/nodejs-npm-buildpack/pull/12))

## [0.1.0] - 2019-10-29

### Added
- feat: use `npm start` as the default launch.toml ([#11](https://github.com/heroku/nodejs-npm-buildpack/pull/11))

## [0.0.2] - 2019-10-11

### Fixed
- Fix broken builds when a `package-lock.json` is missing ([#9](https://github.com/heroku/nodejs-npm-buildpack/pull/9))

[unreleased]: https://github.com/heroku/buildpacks-nodejs/compare/v3.2.13...HEAD
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
