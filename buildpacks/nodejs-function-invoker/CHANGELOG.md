# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

- Drop support for the heroku-20 stack. ([#536](https://github.com/heroku/buildpacks-nodejs/pull/536))

## [0.3.11] 2023/05/22

- Change release target from ECR to docker.io/heroku/buildpack-nodejs-function-invoker.
- Drop explicit support for the End-of-Life stack `heroku-18`.

## [0.3.10] 2023/02/02

- `name` is no longer a required field in package.json. ([#447](https://github.com/heroku/buildpacks-nodejs/pull/447))

## [0.3.9] 2022/12/06
- Update `sf-fx-runtime-nodejs` from `0.14.0` to `0.14.1`

## [0.3.8] 2022/11/30
- Update `sf-fx-runtime-nodejs` from `0.12.0` to `0.14.0` for functions still using the implicit dependency ([#401](https://github.com/heroku/buildpacks-nodejs/pull/401))

## [0.3.7] 2022/10/28
- Fix `sf-fx-runtime-nodejs` dependency installing from `npx` at application startup when implicit runtime dependency is used ([#382](https://github.com/heroku/buildpacks-nodejs/pull/382))

## [0.3.6] 2022/10/26
- Support explicit Functions Runtime for Node.js as dependency in package.json ([#373](https://github.com/heroku/buildpacks-nodejs/pull/373))

## [0.3.5] 2022/09/28

- Update `sf-fx-runtime-nodejs` to `0.12.0`. ([#362](https://github.com/heroku/buildpacks-nodejs/pull/362))
- Upgrade `libcnb` and `libherokubuildpack` to `0.11.0`. ([#360](https://github.com/heroku/buildpacks-nodejs/pull/360))

## [0.3.4] 2022/09/12

- Upgrade `libcnb` and `libherokubuildpack` to `0.10.0`. ([#335](https://github.com/heroku/buildpacks-nodejs/pull/335))
- Buildpack now implements buildpack API version `0.8` and so requires lifecycle version `0.14.x` or newer. ([#335](https://github.com/heroku/buildpacks-nodejs/pull/335))

## [0.3.3] 2022/07/05
- Update `sf-fx-runtime-nodejs` to `0.11.2`

## [0.3.2] 2022/06/29

## [0.3.1] 2022/04/05

- Add support for the heroku-22 stack
- Drop support for the bionic stack

## [0.3.0] 2022/04/01

- Rewrite from bash to libcnb.rs implementation
- Drop /opt/run.sh in favor of direct process entry
- `yj` no longer installed during `detect` and no longer required during `build`

## [0.2.10] 2022/02/23

- Update sf-fx-runtime-nodejs to 0.11.0

## [0.2.9] 2022/02/10

- Update sf-fx-runtime-nodejs to 0.10.0

## [0.2.8] 2022/01/04

- Update sf-fx-runtime-nodejs to 0.9.2

## [0.2.7] 2021/10/18

- Decrease sf-fx-runtime-nodejs workers to 2

## [0.2.6] 2021/10/13

- Update sf-fx-runtime-nodejs to 0.9.1
- Set sf-fx-runtime-nodejs --workers to $WEB_CONCURRENCY

## [0.2.5] 2021/10/13

- Update sf-fx-runtime-nodejs to 0.9.0
- Allow sf-fx-runtime-nodejs to manage it's own --inspect port handling

## [0.2.4] 2021/10/04

- Update sf-fx-runtime-nodejs to 0.8.0

## [0.2.3] 2021/09/30

- Update sf-fx-runtime-nodejs to 0.7.0

## [0.2.2] 2021/09/23

- Update sf-fx-runtime-nodejs to 0.6.0 and install from npmjs.org

## [0.2.1] 2021/09/08

- Update sf-fx-runtime-nodejs to 0.5.2

## [0.2.0] 2021/08/24

- Bump sf-fx-runtime-nodejs to 0.4.0, adding support for JavaScript Modules

## [0.1.7] 2021/07/28

- Bump sf-fx-runtime-nodejs to 0.1.2-ea

## [0.1.6] 2021/06/21
### Changed
- Bump sf-fx-runtime-nodejs to 0.1.1.-ea

## [0.1.5] 2021/05/18
### Fixed
- Use correct path for referencing `lib/utils/download.sh` ([#70](https://github.com/heroku/buildpacks-nodejs/pull/70))

## [0.1.4] 2021/05/18
### Changed
- Detect for `type=function` in `project.toml` ([#58](https://github.com/heroku/buildpacks-nodejs/pull/58))
- Install `yj` before `bin/detect` ([#66](https://github.com/heroku/buildpacks-nodejs/pull/66))

## [0.1.3] 2021/05/12
### Changed
- Fixed `NODE_OPTIONS` unbound variable error when using `DEBUG_PORT` ([#63](https://github.com/heroku/buildpacks-nodejs/pull/63))

## [0.1.2] 2021/05/11
### Added
- Remote debugging is now enabled when the `DEBUG_PORT` environment variable is set ([#59](https://github.com/heroku/buildpacks-nodejs/pull/59))

### Changed
- The `web` process is now marked as the default process type ([#60](https://github.com/heroku/buildpacks-nodejs/pull/60))
- The function runtime download is now cleaned up after installation ([#57](https://github.com/heroku/buildpacks-nodejs/pull/57))

## [0.1.1] 2021/05/10
### Added
- Run check for "main" key and file in package.json ([#52](https://github.com/heroku/buildpacks-nodejs/pull/52))
- Support for newer versions of the function runtime

## [0.1.0] 2021/05/06
### Added
- Initial implementation ([#47](https://github.com/heroku/buildpacks-node/pull/47))
