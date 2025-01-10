# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [3.4.3] - 2025-01-10

- No changes.

## [3.4.2] - 2025-01-08

### Added

- 23.6.0 (linux-amd64, linux-arm64)
- 22.13.0 (linux-amd64, linux-arm64)

## [3.4.1] - 2025-01-07

### Added

- 23.5.0 (linux-amd64, linux-arm64)

## [3.4.0] - 2024-12-13

## [3.3.5] - 2024-12-11

### Added

- 23.4.0 (linux-amd64, linux-arm64)

## [3.3.4] - 2024-12-05

### Added

- 22.12.0 (linux-amd64, linux-arm64)

## [3.3.3] - 2024-11-22

### Added

- 23.3.0 (linux-amd64, linux-arm64)
- 20.18.1 (linux-amd64, linux-arm64)

## [3.3.2] - 2024-11-13

### Added

- 23.2.0 (linux-amd64, linux-arm64)
- 18.20.5 (linux-amd64, linux-arm64)

## [3.3.1] - 2024-11-06

## [3.3.0] - 2024-10-31

### Changed

- Updated default Node.js version to 22.x.
  ([#950](https://github.com/heroku/buildpacks-nodejs/pull/950))

## [3.2.18] - 2024-10-31

### Added

- 22.11.0 (linux-amd64, linux-arm64)

## [3.2.17] - 2024-10-25

### Added

- 23.1.0 (linux-amd64, linux-arm64)

## [3.2.16] - 2024-10-22

### Added

- 23.0.0 (linux-amd64, linux-arm64)
- 22.10.0 (linux-amd64, linux-arm64)

## [3.2.15] - 2024-10-04

### Added

- 20.18.0 (linux-amd64, linux-arm64)

## [3.2.14] - 2024-09-24

### Added

- 22.9.0 (linux-amd64, linux-arm64)

### Fixed

- Fix `heroku/nodejs-engine` layer misconfiguration for `web_env` and
  `node_runtime_metrics`. ([#924](https://github.com/heroku/buildpacks-nodejs/pull/924))

## [3.2.13] - 2024-09-04

### Added

- 22.8.0 (linux-amd64, linux-arm64)

## [3.2.12] - 2024-08-27

### Added

- 22.7.0 (linux-amd64, linux-arm64)
- 20.17.0 (linux-amd64, linux-arm64)

## [3.2.11] - 2024-08-12

### Added

- 22.6.0 (linux-amd64, linux-arm64)

## [3.2.10] - 2024-07-29

### Added

- 20.16.0 (linux-amd64, linux-arm64)

## [3.2.9] - 2024-07-19

### Added

- 22.5.1 (linux-amd64, linux-arm64)

## [3.2.8] - 2024-07-18

### Added

- 22.5.0 (linux-amd64, linux-arm64)

## [3.2.7] - 2024-07-09

### Added

- 22.4.1 (linux-amd64, linux-arm64)
- 20.15.1 (linux-amd64, linux-arm64)
- 18.20.4 (linux-amd64, linux-arm64)

## [3.2.6] - 2024-07-03

### Added

- 22.4.0 (linux-amd64, linux-arm64)

## [3.2.5] - 2024-06-21

### Added

- 20.15.0 (linux-amd64, linux-arm64)

## [3.2.4] - 2024-06-13

### Added

- 22.3.0 (linux-amd64, linux-arm64)

## [3.2.3] - 2024-05-29

### Added

- 20.14.0 (linux-amd64, linux-arm64)

## [3.2.2] - 2024-05-22

### Added

- 22.2.0 (linux-amd64, linux-arm64)
- 18.20.3 (linux-amd64, linux-arm64)

## [3.2.1] - 2024-05-10

### Added

- Node.js 20.13.1 (linux-amd64, linux-arm64)

## [3.2.0] - 2024-05-09

## [3.1.0] - 2024-05-09

### Added

- Support for `arm64` and multi-arch images. ([#815](https://github.com/heroku/buildpacks-nodejs/pull/815))

## [3.0.6] - 2024-05-03

### Added

- Node.js 22.1.0

## [3.0.5] - 2024-04-25

### Added

- Node.js 22.0.0

## [3.0.4] - 2024-04-10

### Added

- Node.js 21.7.3
- Node.js 20.12.2
- Node.js 18.20.2

## [3.0.3] - 2024-04-04

### Added

- Node.js 21.7.2
- Node.js 20.12.1
- Node.js 18.20.1

## [3.0.2] - 2024-03-27

### Added

- Node.js 20.12.0
- Node.js 18.20.0

## [3.0.1] - 2024-03-11

### Added

- Node.js 21.7.1

## [3.0.0] - 2024-03-08

### Added

- Node.js 21.7.0

### Changed

- Bump to Buildpack API 0.10.
  ([#789](https://github.com/heroku/buildpacks-nodejs/pull/789))

## [2.6.6] - 2024-02-15

### Added

- Node.js 21.6.2
- Node.js 20.11.1
- Node.js 18.19.1

## [2.6.5] - 2024-02-01

### Added

- Node.js 21.6.1

### Changed

- Collect Node.js Runtime Metrics for v14.10.0 and up if the application has
  opted-in. ([#767](https://github.com/heroku/buildpacks-nodejs/pull/767))
- Adjusted WEB_MEMORY and WEB_CONCURRENCY calculation to be more appropriate on memory heavy
  instances. ([#764](https://github.com/heroku/buildpacks-nodejs/pull/764))

## [2.6.4] - 2024-01-17

### Added

- Node.js 21.6.0

## [2.6.3] - 2024-01-11

### Added

- Node.js 20.11.0

## [2.6.2] - 2024-01-02

### Added

- Node.js 21.5.0

## [2.6.1] - 2023-12-14

### Added

- Collect Node.js Runtime Metrics if the application has
  opted-in. ([#742](https://github.com/heroku/buildpacks-nodejs/pull/742))

## [2.6.0] - 2023-12-14

## [2.5.0] - 2023-12-07

### Added

- Node.js 21.4.0
- Enabled libcnb `trace` feature, so that OpenTelemetry file exports with
  buildpack detect and build traces are emitted to the file system.
  ([#749](https://github.com/heroku/buildpacks-nodejs/pull/749))

## [2.4.1] - 2023-12-04

## [2.4.0] - 2023-12-01

### Added

- Node.js 21.3.0
- Node.js 21.2.0
- Node.js 20.10.0
- Node.js 18.19.0

## [2.3.0] - 2023-11-09

### Changed

- Updated default node version to 20.x

## [2.2.0] - 2023-10-26

## [2.1.0] - 2023-10-26

### Added

- Node.js 21.1.0
- Node.js 20.9.0

## [2.0.0] - 2023-10-24

### Added

- Node.js 21.0.0

### Changed

- Updated buildpack description and keywords. ([#692](https://github.com/heroku/buildpacks-nodejs/pull/692))

### Removed

- Dropped support for the end of life `io.buildpacks.stacks.bionic`
  stack. ([#693](https://github.com/heroku/buildpacks-nodejs/pull/693))

## [1.1.7] - 2023-10-17

### Added

- Node.js 20.8.1
- Node.js 18.18.2
- Node.js 18.18.1
- Node.js 20.8.0

### Changed

- Provides `npm` added to the build plan since a default version of `npm` is bundled with
  Node.js. ([#622](https://github.com/heroku/buildpacks-nodejs/pull/622))

## [1.1.6] - 2023-09-25

## [1.1.5] - 2023-09-19

### Added

- Node.js 20.7.0
- Node.js 20.6.1
- Node.js 20.6.0
- Node.js 18.18.0

## [1.1.4] - 2023-08-10

### Added

- Node.js 20.5.1
- Node.js 18.17.1
- Node.js 16.20.2

## [1.1.3] - 2023-07-24

### Added

- Node.js 20.5.0

## [1.1.2] - 2023-07-19

### Added

- Node.js 18.17.0

## [1.1.1] - 2023-07-07

### Added

- Node.js 20.4.0

## [1.1.0] - 2023-06-28

## [0.8.24] - 2023-06-21

### Added

- Node.js 20.3.1
- Node.js 18.16.1
- Node.js 16.20.1

## [0.8.23] - 2023-06-14

### Added

- Node.js 20.3.0

### Changed

- Upgrade to Buildpack API version `0.9`. ([#552](https://github.com/heroku/buildpacks-nodejs/pull/552))

## [0.8.22] - 2023-05-22

### Added

- Node.js 20.2.0

### Changed

- Change release target from ECR to docker.io/heroku/buildpack-nodejs-engine.

### Removed

- Drop explicit support for the End-of-Life stack `heroku-18`.

## [0.8.21] - 2023-05-08

### Added

- Node.js 20.1.0

## [0.8.20] - 2023-04-20

### Added

- Node.js 20.0.0

## [0.8.19] - 2023-04-17

### Added

- Node.js 18.16.0

## [0.8.18] - 2023-04-12

### Added

- Node.js 19.9.0

## [0.8.17] - 2023-04-03

### Added

- Node.js 19.8.1
- Node.js 19.8.0
- Node.js 18.15.0
- Node.js 16.20.0

## [0.8.16] - 2023-02-27

### Added

- Node.js 19.7.0
- Node.js 19.6.1
- Node.js 19.6.0
- Node.js 18.14.1
- Node.js 18.14.2
- Node.js 18.14.0
- Node.js 16.19.1
- Node.js 14.21.3

## [0.8.15] - 2023-02-02

### Added

- Node.js 19.5.0

### Changed

- `name` is no longer a required field in package.json. ([#447](https://github.com/heroku/buildpacks-nodejs/pull/447))

## [0.8.14] - 2023-01-17

### Added

- Node.js 19.4.0
- Node.js 19.3.0
- Node.js 18.13.0
- Node.js 16.19.0
- Node.js 14.21.2

## [0.8.13] - 2022-12-05

### Added

- Node.js 19.2.0
- Node.js 19.1.0

## [0.8.12] - 2022-11-04

### Added

- Node.js 19.0.1
- Node.js 18.12.1
- Node.js 16.18.1
- Node.js 14.21.1
- Node.js 14.21.0

## [0.8.11] - 2022-11-01

### Changed

- Don't overwrite WEB_CONCURRENCY if already set. ([#386](https://github.com/heroku/buildpacks-nodejs/pull/386))

## [0.8.10] - 2022-10-28

### Added

- Node.js 19.0.0
- Node.js 18.12.0
- Node.js 18.11.0
- Node.js 18.10.0
- Node.js 16.18.0

## [0.8.9] - 2022-09-28

### Added

- Node.js 18.9.1
- Node.js 16.17.1
- Node.js 14.20.1

### Changed

- Upgrade `libcnb` and `libherokubuildpack` to `0.11.0`. ([#360](https://github.com/heroku/buildpacks-nodejs/pull/360))

## [0.8.8] - 2022-09-12

### Added

- Node.js 18.9.0
- Node.js 18.8.0
- Node.js 18.6.0
- Node.js 18.7.0
- Node.js 16.17.0

### Changed

- Upgrade `libcnb` and `libherokubuildpack` to `0.10.0`. ([#335](https://github.com/heroku/buildpacks-nodejs/pull/335))
- Buildpack now implements buildpack API version `0.8` and so requires lifecycle version `0.14.x` or
  newer. ([#335](https://github.com/heroku/buildpacks-nodejs/pull/335))

## [0.8.7] - 2022-07-12

### Added

- Node.js 18.5.0
- Node.js 18.4.0
- Node.js 16.16.0
- Node.js 14.20.0

### Changed

- Bump libcnb to 0.8.0. ([#286](https://github.com/heroku/buildpacks-nodejs/pull/286)).

## [0.8.6] - 2022-06-14

### Changed

- Switch away from deprecated path-based S3 URLs

## [0.8.5] - 2022-06-08

### Added

- Node.js 18.3.0
- Node.js 17.9.1
- Node.js 16.15.1

## [0.8.4] - 2022-05-23

### Added

- Node.js 18.2.0
- Node.js 18.1.0
- Node.js 18.0.0
- Node.js 17.9.0
- Node.js 16.15.0
- Node.js 14.19.3
- Node.js 14.19.2
- Node.js 12.22.12

## [0.8.3] - 2022-04-05

### Added

- Add support for the heroku-22 stack

## [0.8.2] - 2022-04-01

### Changed

- Update Node.js inventory ([#225](https://github.com/heroku/buildpacks-nodejs/pull/225))

## [0.8.1] - 2022-03-23

### Changed

- `package.json`'s `version` field is now optional ([#215](https://github.com/heroku/buildpacks-nodejs/pull/215))

## [0.8.0] - 2022-03-09

### Changed

- Convert buildpack from bash to rust leveraging
  libcnb.rs ([#184](https://github.com/heroku/buildpacks-nodejs/pull/184))
- Now conditionally `requires` node, making the buildpack independently
  usable ([#184](https://github.com/heroku/buildpacks-nodejs/pull/184))
- Replaces go-based version resolver with rust
  implementation ([#184](https://github.com/heroku/buildpacks-nodejs/pull/184))
- Replaces bash based WEB_CONCURRENCY profile.d script with rust / exec.d
  implementation ([#184](https://github.com/heroku/buildpacks-nodejs/pull/184))

### Removed

- No longer installs `yarn`, that is now a function
  of `heroku/nodejs-yarn` ([#184](https://github.com/heroku/buildpacks-nodejs/pull/184))
- No longer installs `yq` or the toolbox build layer ([#184](https://github.com/heroku/buildpacks-nodejs/pull/184))

## [0.7.5] - 2022-01-28

### Fixed

- Ensure NODE_ENV is set consistently during build, no matter the cache
  state ([186](https://github.com/heroku/buildpacks-nodejs/pull/186)

## [0.7.4] - 2021-06-15

### Changed

- Change node engine version from 12 to 14 ([#40](https://github.com/heroku/buildpacks-node/pull/40))
- Clear cache when node version changes ([#40](https://github.com/heroku/buildpacks-node/pull/40))
- Check for nodejs.toml before read ([#53](https://github.com/heroku/buildpacks-nodejs/pull/53))
- Change default Node.js version to 16 ([#53](https://github.com/heroku/buildpacks-nodejs/pull/53))

### Fixed

- Fix bug that causes an error on Node version change ([#77](https://github.com/heroku/buildpacks-nodejs/pull/77))

## [0.7.3] - 2021-03-04

### Changed

- Flush cache when stack image changes ([#28](https://github.com/heroku/buildpacks-node/pull/28))
- Trim whitespace when getting stack name ([#29](https://github.com/heroku/buildpacks-node/pull/29))

## [0.7.2] - 2021-02-23

### Added

- Add license to buildpack.toml ([#17](https://github.com/heroku/buildpacks-node/pull/17))

### Changed

- Copy node modules directory path into the build ENV ([#15](https://github.com/heroku/buildpacks-node/pull/15))

### Removed

- Remove package.json requirement ([#14](https://github.com/heroku/buildpacks-node/pull/14))

## [0.7.1] - 2021-01-20

### Changed

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

- Increase `MaxKeys` for listing S3 objects in `resolve-version`
  query ([#43](https://github.com/heroku/nodejs-engine-buildpack/pull/43))
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

- Remove catching of unbound variables
  in `lib/build.sh` ([#36](https://github.com/heroku/nodejs-engine-buildpack/pull/36))

## [0.4.2] - 2020-01-30

### Added

- Write bootstrapped binaries to a layer instead of to `bin`; Add a logging method for build
  output ([#34](https://github.com/heroku/nodejs-engine-buildpack/pull/34))
- Added `provides` and `requires` of `node` to
  buildplan. ([#31](https://github.com/heroku/nodejs-engine-buildpack/pull/31))

## [0.4.1] - 2019-11-08

### Fixed

- Fix updates to `nodejs.toml` when layer contents not
  updated ([#27](https://github.com/heroku/nodejs-engine-buildpack/pull/27))

## [0.4.0] - 2019-10-31

### Added

- Add launch.toml support to engine ([#26](https://github.com/heroku/nodejs-engine-buildpack/pull/26))
- Parse engines and add them to nodejs.toml ([#25](https://github.com/heroku/nodejs-engine-buildpack/pull/25))
- Add shellcheck to test suite ([#24](https://github.com/heroku/nodejs-engine-buildpack/pull/24))

[unreleased]: https://github.com/heroku/buildpacks-nodejs/compare/v3.4.3...HEAD
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
