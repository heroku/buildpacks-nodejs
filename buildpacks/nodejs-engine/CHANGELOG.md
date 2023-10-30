# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Update default node version to 20.x

## [2.2.0] - 2023-10-26

- No changes.

## [2.1.0] - 2023-10-26

### Added

- Added Node.js version 21.1.0.
- Added Node.js version 20.9.0.

## [2.0.0] - 2023-10-24

### Added

- Added Node.js version 21.0.0.

### Changed

- Updated buildpack description and keywords. ([#692](https://github.com/heroku/buildpacks-nodejs/pull/692))

### Removed

- Dropped support for the end of life `io.buildpacks.stacks.bionic` stack. ([#693](https://github.com/heroku/buildpacks-nodejs/pull/693))

## [1.1.7] - 2023-10-17

- Added Node.js version 20.8.1.
- Added Node.js version 18.18.2.
- Added Node.js version 18.18.1.
- Added Node.js version 20.8.0.
- Provides `npm` added to the build plan since a default version of `npm` is bundled with Node.js. ([#622](https://github.com/heroku/buildpacks-nodejs/pull/622))

## [1.1.6] - 2023-09-25

- No changes.

## [1.1.5] - 2023-09-19

- Added Node.js version 20.7.0.
- Added Node.js version 18.18.0.
- Added Node.js version 20.6.1.
- Added Node.js version 20.6.0.

## [1.1.4] - 2023-08-10

- Added Node.js version 16.20.2.
- Added Node.js version 18.17.1.
- Added Node.js version 20.5.1.

## [1.1.3] - 2023-07-24

- Added Node.js version 20.5.0.

## [1.1.2] - 2023-07-19

- Added Node.js version 18.17.0.

## [1.1.1] - 2023-07-07

- Added Node.js version 20.4.0.

## [1.1.0] - 2023-06-28

- No changes

## [0.8.24] - 2023-06-21

- Added Node.js version 20.3.1, 18.16.1, 16.20.1.

## [0.8.23] - 2023-06-14

- Added Node.js version 20.3.0.
- Upgrade to Buildpack API version `0.9`. ([#552](https://github.com/heroku/buildpacks-nodejs/pull/552))

## [0.8.22] - 2023-05-22

- Change release target from ECR to docker.io/heroku/buildpack-nodejs-engine.
- Drop explicit support for the End-of-Life stack `heroku-18`.
- Added node version 20.2.0.

## [0.8.21] - 2023-05-08

- Added node version 20.1.0.

## [0.8.20] - 2023-04-20

- Added node version 20.0.0.

## [0.8.19] - 2023-04-17

- Added node version 18.16.0.

## [0.8.18] - 2023-04-12

- Added node version 19.9.0.

## [0.8.17] - 2023-04-03

- Added node version 16.20.0.
- Added node version 19.8.1, 19.8.0.
- Added node version 18.15.0.

## [0.8.16] - 2023-02-27

- Added node version 19.7.0, 19.6.1, 14.21.3, 16.19.1, 18.14.1, 18.14.2.
- Added node version 18.14.0, 19.6.0.

## [0.8.15] - 2023-02-02

- `name` is no longer a required field in package.json. ([#447](https://github.com/heroku/buildpacks-nodejs/pull/447))
- Added node version 19.5.0.

## [0.8.14] - 2023-01-17

- Added node version 18.13.0, 19.4.0.
- Added node version 19.3.0, 16.19.0, 14.21.2.

## [0.8.13] - 2022-12-05

- Added node version 19.2.0.
- Added node version 19.1.0.

## [0.8.12] - 2022-11-04

- Added node version 19.0.1, 14.21.1, 18.12.1, 16.18.1.
- Added node version 14.21.0.

## [0.8.11] - 2022-11-01

- Don't overwrite WEB_CONCURRENCY if already set. ([#386](https://github.com/heroku/buildpacks-nodejs/pull/386))

## [0.8.10] - 2022-10-28

- Added node version 18.12.0.
- Added node version 19.0.0.
- Added node version 16.18.0, 18.11.0, 18.10.0.

## [0.8.9] - 2022-09-28

- Added node version 14.20.1, 18.9.1, 16.17.1.
- Upgrade `libcnb` and `libherokubuildpack` to `0.11.0`. ([#360](https://github.com/heroku/buildpacks-nodejs/pull/360))

## [0.8.8] - 2022-09-12

- Added node version 18.9.0.
- Added node version 18.8.0.
- Added node version 16.17.0.
- Added node version 18.6.0, 18.7.0.
- Upgrade `libcnb` and `libherokubuildpack` to `0.10.0`. ([#335](https://github.com/heroku/buildpacks-nodejs/pull/335))
- Buildpack now implements buildpack API version `0.8` and so requires lifecycle version `0.14.x` or newer. ([#335](https://github.com/heroku/buildpacks-nodejs/pull/335))

## [0.8.7] - 2022-07-12

- Added node version 14.20.0, 18.5.0, 16.16.0.
- Added node version 18.4.0.
- Bump libcnb to 0.8.0. ([#286](https://github.com/heroku/buildpacks-nodejs/pull/286)).

## [0.8.6] - 2022-06-14

- Switch away from deprecated path-based S3 URLs

## [0.8.5] - 2022-06-08

- Added node version 16.15.1, 18.3.0, 17.9.1.

## [0.8.4] - 2022-05-23

- Added node version 14.19.3, 18.2.0.
- Added node version 14.19.2, 18.1.0, 16.15.0.
- Added node version 18.0.0.
- Added node version 17.9.0.
- Added node version 12.22.12.

## [0.8.3] - 2022-04-05

- Add support for the heroku-22 stack

## [0.8.2] - 2022-04-01

- Update Node.js inventory ([#225](https://github.com/heroku/buildpacks-nodejs/pull/225))

## [0.8.1] - 2022-03-23

- `package.json`'s `version` field is now optional ([#215](https://github.com/heroku/buildpacks-nodejs/pull/215))

## [0.8.0] - 2022-03-09

- Convert buildpack from bash to rust leveraging libcnb.rs ([#184](https://github.com/heroku/buildpacks-nodejs/pull/184))
- Now conditionally `requires` node, making the buildpack independently usable ([#184](https://github.com/heroku/buildpacks-nodejs/pull/184))
- No longer installs `yarn`, that is now a function of `heroku/nodejs-yarn` ([#184](https://github.com/heroku/buildpacks-nodejs/pull/184))
- No longer installs `yq` or the toolbox build layer ([#184](https://github.com/heroku/buildpacks-nodejs/pull/184))
- Replaces go-based version resolver with rust implementation ([#184](https://github.com/heroku/buildpacks-nodejs/pull/184))
- Replaces bash based WEB_CONCURRENCY profile.d script with rust / exec.d implementation ([#184](https://github.com/heroku/buildpacks-nodejs/pull/184))

## [0.7.5] - 2022-01-28

- Ensure NODE_ENV is set consistently during build, no matter the cache state ([186](https://github.com/heroku/buildpacks-nodejs/pull/186)

## [0.7.4] - 2021-06-15

- Change node engine version from 12 to 14 ([#40](https://github.com/heroku/buildpacks-node/pull/40))
- Clear cache when node version changes ([#40](https://github.com/heroku/buildpacks-node/pull/40))
- Check for nodejs.toml before read ([#53](https://github.com/heroku/buildpacks-nodejs/pull/53))
- Change default Node.js version to 16 ([#53](https://github.com/heroku/buildpacks-nodejs/pull/53))
- Fix bug that causes an error on Node version change ([#77](https://github.com/heroku/buildpacks-nodejs/pull/77))

## [0.7.3] - 2021-03-04

- Flush cache when stack image changes ([#28](https://github.com/heroku/buildpacks-node/pull/28))
- Trim whitespace when getting stack name ([#29](https://github.com/heroku/buildpacks-node/pull/29))

## [0.7.2] - 2021-02-23

- Add license to buildpack.toml ([#17](https://github.com/heroku/buildpacks-node/pull/17))
- Copy node modules directory path into the build ENV ([#15](https://github.com/heroku/buildpacks-node/pull/15))
- Remove package.json requirement ([#14](https://github.com/heroku/buildpacks-node/pull/14))

## [0.7.1] - 2021-01-20

- Replace logging style to match style guides ([#63](https://github.com/heroku/nodejs-engine-buildpack/pull/63))
- Change log colors to use ANSI codes ([#65](https://github.com/heroku/nodejs-engine-buildpack/pull/65))

## [0.7.0] - 2020-11-11

### Added
- Add support for heroku-20 ([#60](https://github.com/heroku/nodejs-engine-buildpack/pull/60))

### Fixed
- Remove jq installation ([#57](https://github.com/heroku/nodejs-engine-buildpack/pull/57))
- Make `NODE_ENV` variables overrides ([#61](https://github.com/heroku/nodejs-engine-buildpack/pull/61))

## [0.6.0] - 2020-10-13

### Added
- Add profile.d script ([#53](https://github.com/heroku/nodejs-engine-buildpack/pull/53))
- Set NODE_ENV to production at runtime ([#54](https://github.com/heroku/nodejs-engine-buildpack/pull/54))
- Set NODE_ENV in build environment ([#55](https://github.com/heroku/nodejs-engine-buildpack/pull/55))

## [0.5.0] - 2020-07-16

### Added
- Increase `MaxKeys` for listing S3 objects in `resolve-version` query ([#43](https://github.com/heroku/nodejs-engine-buildpack/pull/43))
- Add Circle CI test integration ([#49](https://github.com/heroku/nodejs-engine-buildpack/pull/49))

## [0.4.4] - 2020-03-25

### Added
- Add shpec to shellcheck ([#38](https://github.com/heroku/nodejs-engine-buildpack/pull/38))
- Dockerize unit tests with shpec ([#39](https://github.com/heroku/nodejs-engine-buildpack/pull/39))

### Fixed
- Upgrade Go version to 1.14 ([#40](https://github.com/heroku/nodejs-engine-buildpack/pull/40))
- Use cached bootstrap binaries when present ([#42](https://github.com/heroku/nodejs-engine-buildpack/pull/42))

## [0.4.3] - 2020-02-24

### Fixed
- Remove catching of unbound variables in `lib/build.sh` ([#36](https://github.com/heroku/nodejs-engine-buildpack/pull/36))

## [0.4.2] - 2020-01-30

### Added
- Write bootstrapped binaries to a layer instead of to `bin`; Add a logging method for build output ([#34](https://github.com/heroku/nodejs-engine-buildpack/pull/34))
- Added `provides` and `requires` of `node` to buildplan. ([#31](https://github.com/heroku/nodejs-engine-buildpack/pull/31))

## [0.4.1] - 2019-11-08

### Fixed
- Fix updates to `nodejs.toml` when layer contents not updated ([#27](https://github.com/heroku/nodejs-engine-buildpack/pull/27))

## [0.4.0] - 2019-10-31

### Added
- Add launch.toml support to engine ([#26](https://github.com/heroku/nodejs-engine-buildpack/pull/26))
- Parse engines and add them to nodejs.toml ([#25](https://github.com/heroku/nodejs-engine-buildpack/pull/25))
- Add shellcheck to test suite ([#24](https://github.com/heroku/nodejs-engine-buildpack/pull/24))

[unreleased]: https://github.com/heroku/buildpacks-nodejs/compare/v2.2.0...HEAD
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
