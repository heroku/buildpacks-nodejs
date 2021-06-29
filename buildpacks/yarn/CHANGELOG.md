# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [0.1.5] 2021/06/17
### Fixed
- Empty cache builds no longer fail with a `PREV_NODE_VERSION ` unbound variable error ([#86](https://github.com/heroku/buildpacks-node/pull/86))
- yarn buildpack consumes dependency on node during plan resolution

## [0.1.4] 2021/06/15
### Fixed
- Clear cache when node version changes ([#40](https://github.com/heroku/buildpacks-node/pull/40))

## [0.1.3] 2021/03/04
- Add license to buildpack.toml ([#17](https://github.com/heroku/buildpacks-node/pull/17))
- Flush cache when stack image changes ([#28](https://github.com/heroku/buildpacks-node/pull/28))
- Trim whitespace when getting stack name ([#29](https://github.com/heroku/buildpacks-node/pull/29))
- Fail if two lock files are detected ([#30](https://github.com/heroku/buildpacks-node/pull/30))

## [0.1.1] 2021/01/20

## [0.1.0] 2020/11/11
### Added
- Add support for heroku-20 and bionic stacks ([#4](https://github.com/heroku/nodejs-yarn-buildpack/pull/4))

## [0.0.1] 2019/12/08
### Added
- Changelog entry for first release ([#1](https://github.com/heroku/nodejs-yarn-buildpack/pull/1))
