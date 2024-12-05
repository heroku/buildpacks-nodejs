# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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

- No changes.

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

- No changes.

## [3.2.2] - 2024-05-22

### Changed

- Update function runtime to 0.14.5 ([#845](https://github.com/heroku/buildpacks-nodejs/pull/845))

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

- No changes.

## [2.4.1] - 2023-12-04

### Changed

- Update function runtime to 0.14.4 ([#734](https://github.com/heroku/buildpacks-nodejs/pull/734))

## [2.4.0] - 2023-12-01

- No changes.

## [2.3.0] - 2023-11-09

- No changes.

## [2.2.0] - 2023-10-26

- No changes.

## [2.1.0] - 2023-10-26

- No changes.

## [2.0.0] - 2023-10-24

### Changed

- Updated buildpack display name, description and keywords. ([#692](https://github.com/heroku/buildpacks-nodejs/pull/692))

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

## [0.3.12] - 2023-06-14

- Upgrade to Buildpack API version `0.9`. ([#552](https://github.com/heroku/buildpacks-nodejs/pull/552))
- Drop support for the heroku-20 stack. ([#536](https://github.com/heroku/buildpacks-nodejs/pull/536))

## [0.3.11] - 2023-05-22

- Change release target from ECR to docker.io/heroku/buildpack-nodejs-function-invoker.
- Drop explicit support for the End-of-Life stack `heroku-18`.

## [0.3.10] - 2023-02-02

- `name` is no longer a required field in package.json. ([#447](https://github.com/heroku/buildpacks-nodejs/pull/447))

## [0.3.9] - 2022-12-06

- Update `sf-fx-runtime-nodejs` from `0.14.0` to `0.14.1`

## [0.3.8] - 2022-11-30

- Update `sf-fx-runtime-nodejs` from `0.12.0` to `0.14.0` for functions still using the implicit dependency ([#401](https://github.com/heroku/buildpacks-nodejs/pull/401))

## [0.3.7] - 2022-10-28

- Fix `sf-fx-runtime-nodejs` dependency installing from `npx` at application startup when implicit runtime dependency is used ([#382](https://github.com/heroku/buildpacks-nodejs/pull/382))

## [0.3.6] - 2022-10-26

- Support explicit Functions Runtime for Node.js as dependency in package.json ([#373](https://github.com/heroku/buildpacks-nodejs/pull/373))

## [0.3.5] - 2022-09-28

- Update `sf-fx-runtime-nodejs` to `0.12.0`. ([#362](https://github.com/heroku/buildpacks-nodejs/pull/362))
- Upgrade `libcnb` and `libherokubuildpack` to `0.11.0`. ([#360](https://github.com/heroku/buildpacks-nodejs/pull/360))

## [0.3.4] - 2022-09-12

- Upgrade `libcnb` and `libherokubuildpack` to `0.10.0`. ([#335](https://github.com/heroku/buildpacks-nodejs/pull/335))
- Buildpack now implements buildpack API version `0.8` and so requires lifecycle version `0.14.x` or newer. ([#335](https://github.com/heroku/buildpacks-nodejs/pull/335))

## [0.3.3] - 2022-07-05

- Update `sf-fx-runtime-nodejs` to `0.11.2`

## [0.3.2] - 2022-06-29

## [0.3.1] - 2022-04-05

- Add support for the heroku-22 stack
- Drop support for the bionic stack

## [0.3.0] - 2022-04-01

- Rewrite from bash to libcnb.rs implementation
- Drop /opt/run.sh in favor of direct process entry
- `yj` no longer installed during `detect` and no longer required during `build`

## [0.2.10] - 2022-02-23

- Update sf-fx-runtime-nodejs to 0.11.0

## [0.2.9] - 2022-02-10

- Update sf-fx-runtime-nodejs to 0.10.0

## [0.2.8] - 2022-01-04

- Update sf-fx-runtime-nodejs to 0.9.2

## [0.2.7] - 2021-10-18

- Decrease sf-fx-runtime-nodejs workers to 2

## [0.2.6] - 2021-10-13

- Update sf-fx-runtime-nodejs to 0.9.1
- Set sf-fx-runtime-nodejs --workers to $WEB_CONCURRENCY

## [0.2.5] - 2021-10-13

- Update sf-fx-runtime-nodejs to 0.9.0
- Allow sf-fx-runtime-nodejs to manage it's own --inspect port handling

## [0.2.4] - 2021-10-04

- Update sf-fx-runtime-nodejs to 0.8.0

## [0.2.3] - 2021-09-30

- Update sf-fx-runtime-nodejs to 0.7.0

## [0.2.2] - 2021-09-23

- Update sf-fx-runtime-nodejs to 0.6.0 and install from npmjs.org

## [0.2.1] - 2021-09-08

- Update sf-fx-runtime-nodejs to 0.5.2

## [0.2.0] - 2021-08-24

- Bump sf-fx-runtime-nodejs to 0.4.0, adding support for JavaScript Modules

## [0.1.7] - 2021-07-28

- Bump sf-fx-runtime-nodejs to 0.1.2-ea

## [0.1.6] - 2021-06-21

### Changed
- Bump sf-fx-runtime-nodejs to 0.1.1.-ea

## [0.1.5] - 2021-05-18

### Fixed
- Use correct path for referencing `lib/utils/download.sh` ([#70](https://github.com/heroku/buildpacks-nodejs/pull/70))

## [0.1.4] - 2021-05-18

### Changed
- Detect for `type=function` in `project.toml` ([#58](https://github.com/heroku/buildpacks-nodejs/pull/58))
- Install `yj` before `bin/detect` ([#66](https://github.com/heroku/buildpacks-nodejs/pull/66))

## [0.1.3] - 2021-05-12

### Changed
- Fixed `NODE_OPTIONS` unbound variable error when using `DEBUG_PORT` ([#63](https://github.com/heroku/buildpacks-nodejs/pull/63))

## [0.1.2] - 2021-05-11

### Added
- Remote debugging is now enabled when the `DEBUG_PORT` environment variable is set ([#59](https://github.com/heroku/buildpacks-nodejs/pull/59))

### Changed
- The `web` process is now marked as the default process type ([#60](https://github.com/heroku/buildpacks-nodejs/pull/60))
- The function runtime download is now cleaned up after installation ([#57](https://github.com/heroku/buildpacks-nodejs/pull/57))

## [0.1.1] - 2021-05-10

### Added
- Run check for "main" key and file in package.json ([#52](https://github.com/heroku/buildpacks-nodejs/pull/52))
- Support for newer versions of the function runtime

## [0.1.0] - 2021-05-06

### Added
- Initial implementation ([#47](https://github.com/heroku/buildpacks-node/pull/47))

[unreleased]: https://github.com/heroku/buildpacks-nodejs/compare/v3.3.4...HEAD
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
