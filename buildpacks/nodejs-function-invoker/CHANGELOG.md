# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

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
