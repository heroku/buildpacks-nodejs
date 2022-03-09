# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
* Upgraded `heroku/nodejs-engine` to `0.8.0`

## [0.4.2] 2022/03/09
* Upgraded `heroku/nodejs-npm` to `0.5.0`
* Upgraded `heroku/nodejs-yarn` to `0.2.0`

## [0.4.1] 2022/01/28
* Upgraded `heroku/nodejs-engine` to `0.7.5`
* Upgraded `heroku/nodejs-npm` to `0.4.5`

## [0.4.0] 2022/01/19
* Drop `heroku/nodejs-typescript`. Typescript is fully supported via `heroku/nodejs-yarn` and `heroku/nodejs-npm`.
* Upgraded `heroku/nodejs-yarn` to `0.1.8`

## [0.3.8] 2021/09/29

## [0.3.7] 2021/08/04
* Upgraded `heroku/nodejs-yarn` to `0.1.6`

## [0.3.6] 2021/06/17
* Upgraded `heroku/nodejs-yarn` to `0.1.5`

## [0.3.5] 2021/06/15
* Upgraded `heroku/nodejs-typescript` to `0.2.3`
* Upgraded `heroku/nodejs-yarn` to `0.1.4`
* Upgraded `heroku/nodejs-npm` to `0.4.4`
* Upgraded `heroku/nodejs-engine` to `0.7.4`

## [0.3.4] 2021/05/18

## [0.3.3] 2021/03/10
* Update `heroku/procfile` to reference tag

## [0.3.2] 2021/03/08
* Upgraded `heroku/nodejs-typescript` to `0.2.2`
* Upgraded `heroku/nodejs-yarn` to `0.1.3`
* Upgraded `heroku/nodejs-npm` to `0.4.3`
* Upgraded `heroku/nodejs-engine` to `0.7.3`
### Changed
* Upgrade `heroku/procfile` to `0.6.2`

## [0.3.0] 2021/02/26

## [0.2.0] 2021/02/23
* Upgraded `heroku/nodejs-typescript` to `0.2.1`
* Upgraded `heroku/nodejs-npm` to `0.4.2`
* Upgraded `heroku/nodejs-yarn` to `0.1.2`
* Upgraded `heroku/nodejs-engine` to `0.7.2`
### Added
* Add license to buildpack.toml ([#17](https://github.com/heroku/buildpacks-node/pull/17))
* Automated post-release PRs
### Changed
* Package meta buildpack with latest releases of buildpacks while testing against unreleased.

## [0.1.1]
* Initial release
