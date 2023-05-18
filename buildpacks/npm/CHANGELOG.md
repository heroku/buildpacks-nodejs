# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

- Drop explicit support for the End-of-Life stack `heroku-18`.

## [0.5.2] 2022/04/05

- Add support for all stacks
- Add explicit support for the heroku-22 stack

## [0.5.1] 2022/03/23

- The `web` process affiliated with `package.json`'s `scripts.start` is now a `default` process ([#214](https://github.com/heroku/buildpacks-nodejs/pull/214))

## [0.5.0] 2022/03/09

- Upgraded to buildpack api 0.6 ([#184](https://github.com/heroku/buildpacks-nodejs/pull/184))
- Installs `yq` in the build toolbox layer ([#184](https://github.com/heroku/buildpacks-nodejs/pull/184))

## [0.4.5] 2022/01/28
- `npm ci` and `npm install` now run with `--production=false` to ensure `devDependencies` are installed ([186](https://github.com/heroku/buildpacks-nodejs/pull/186))

## [0.4.4] 2021/06/15
### Fixed
- Clear cache when node version changes ([#40](https://github.com/heroku/buildpacks-node/pull/40))

## [0.4.3] 2021/03/04
- Flush cache when stack image changes ([#28](https://github.com/heroku/buildpacks-node/pull/28))
- Trim whitespace when getting stack name ([#29](https://github.com/heroku/buildpacks-node/pull/29))
- Fail if two lock files are detected ([#30](https://github.com/heroku/buildpacks-node/pull/30))

## [0.4.2] 2021/02/23
- Add license to buildpack.toml ([#17](https://github.com/heroku/buildpacks-node/pull/17))

## [0.4.1] 2021/01/20
- Ensure prefix directory exists ([#42](https://github.com/heroku/nodejs-npm-buildpack/pull/44))
- Use new logging style ([#45](https://github.com/heroku/nodejs-npm-buildpack/pull/45))
- Change log colors to use ANSI codes ([#47](https://github.com/heroku/nodejs-npm-buildpack/pull/47))

## [0.4.0] 2020/11/11
### Added
- Add heroku-20 to supported stacks ([#40](https://github.com/heroku/nodejs-npm-buildpack/pull/40))

## [0.3.0] 2020/09/16
### Added
- Prune devdependencies ([#32](https://github.com/heroku/nodejs-npm-buildpack/pull/32))
- Opt out of pruning devdependencies if NODE_ENV is not production ([#33](https://github.com/heroku/nodejs-npm-buildpack/pull/33))
- Warn when node modules are checked into git ([#34](https://github.com/heroku/nodejs-npm-buildpack/pull/34))
- Add logging method for warnings ([#35](https://github.com/heroku/nodejs-npm-buildpack/pull/35))
### Fixed
- Move integration testing to CirleCI ([#37](https://github.com/heroku/nodejs-npm-buildpack/pull/37))

## [0.2.0] 2020/05/19
### Added
- docs: add docs around `Permission denied` issues ([#28](https://github.com/heroku/nodejs-npm-buildpack/pull/28))
- Add dockerized unit tests ([#29](https://github.com/heroku/nodejs-npm-buildpack/pull/29))
- Added `provides` and `requires` of `node_modules` and `node` to buildplan. ([#18](https://github.com/heroku/nodejs-npm-buildpack/pull/18))

## [0.1.4] 2020/02/19
### Added
- feat: install `npm` version specified in `package.json` ([#24](https://github.com/heroku/nodejs-npm-buildpack/pull/24))
- feat: exchange echo commands for `log_info` method ([#25](https://github.com/heroku/nodejs-npm-buildpack/pull/25))
### Fixed
- fix: use_npm_ci expression return value id ([#22](https://github.com/heroku/nodejs-npm-buildpack/pull/23))

## [0.1.3] 2020/01/28
### Fixed
- fix: remove `-buildpack` from buildpack id ([#16](https://github.com/heroku/nodejs-npm-buildpack/pull/16))
- feat: support running on `io.buildpacks.stacks.bionic` stack ([#17](https://github.com/heroku/nodejs-npm-buildpack/pull/17))

## [0.1.2] 2019/11/01
### Added
- feat: support build time environment variables ([#14](https://github.com/heroku/nodejs-npm-buildpack/pull/14))

## [0.1.1] 2019/10/30
### Fixed
- Fix copying node_modules when a `package-lock.json` is present ([#12](https://github.com/heroku/nodejs-npm-buildpack/pull/12))

## [0.1.0] 2019/10/29
### Added
- feat: use `npm start` as the default launch.toml ([#11](https://github.com/heroku/nodejs-npm-buildpack/pull/11))

## [0.0.2] 2019/10/11
### Fixed
- Fix broken builds when a `package-lock.json` is missing ([#9](https://github.com/heroku/nodejs-npm-buildpack/pull/9))
