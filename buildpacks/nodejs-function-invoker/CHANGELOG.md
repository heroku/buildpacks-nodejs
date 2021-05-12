# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
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
