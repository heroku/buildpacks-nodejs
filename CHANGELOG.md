# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [5.2.0] - 2025-10-27

### Added

- Support for the `engines.pnpm` field in `package.json` to declare pnpm as the package manager. ([#1203](https://github.com/heroku/buildpacks-nodejs/pull/1203))

### Changed

- The `corepack` tool will no longer be used to install pnpm. Usage of the `packageManager` field in `package.json` to declare pnpm as the package manager is still supported. ([#1203](https://github.com/heroku/buildpacks-nodejs/pull/1203))
- The `corepack` tool will no longer be used to install Yarn. Usage of the `packageManager` field in `package.json` to declare Yarn as the package manager is still supported. ([#1204](https://github.com/heroku/buildpacks-nodejs/pull/1204))
- The default version of `yarn@1.22.x` will no longer be installed to bootstrap vendored Yarn installations configured using `yarnPath` in `.yarnrc.yml`. ([#1204](https://github.com/heroku/buildpacks-nodejs/pull/1204))

## [5.1.4] - 2025-10-23

### Added

- 25.0.0 (linux-amd64, linux-arm64)
- 22.21.0 (linux-amd64, linux-arm64)

## [5.1.3] - 2025-10-09

### Added

- 24.10.0 (linux-amd64, linux-arm64)

## [5.1.2] - 2025-09-26

### Added

- 24.9.0 (linux-amd64, linux-arm64)

## [5.1.1] - 2025-09-25

### Added

- 22.20.0 (linux-amd64, linux-arm64)

## [5.1.0] - 2025-09-12

### Added

- 24.8.0 (linux-amd64, linux-arm64)

## [5.0.1] - 2025-09-04

### Added

- 20.19.5 (linux-amd64, linux-arm64)

## [5.0.0] - 2025-09-03

### Changed

- Merged functionality from individual buildpacks into `heroku/nodejs`, previous `CHANGELOG.md` files are linked below ([#1169](https://github.com/heroku/buildpacks-nodejs/pull/1169)):
  - [heroku/nodejs-corepack](https://github.com/heroku/buildpacks-nodejs/blob/v4.1.4/buildpacks/nodejs-corepack/CHANGELOG.md)
  - [heroku/nodejs-engine](https://github.com/heroku/buildpacks-nodejs/blob/v4.1.4/buildpacks/nodejs-engine/CHANGELOG.md)
  - [heroku/nodejs-npm-engine](https://github.com/heroku/buildpacks-nodejs/blob/v4.1.4/buildpacks/nodejs-npm-engine/CHANGELOG.md)
  - [heroku/nodejs-npm-install](https://github.com/heroku/buildpacks-nodejs/blob/v4.1.4/buildpacks/nodejs-npm-install/CHANGELOG.md)
  - [heroku/nodejs-pnpm-engine](https://github.com/heroku/buildpacks-nodejs/blob/v4.1.4/buildpacks/nodejs-pnpm-engine/CHANGELOG.md)
  - [heroku/nodejs-pnpm-install](https://github.com/heroku/buildpacks-nodejs/blob/v4.1.4/buildpacks/nodejs-pnpm-install/CHANGELOG.md)
  - [heroku/nodejs-yarn](https://github.com/heroku/buildpacks-nodejs/blob/v4.1.4/buildpacks/nodejs-yarn/CHANGELOG.md)
- Updated provides to `heroku/nodejs`. ([#1169](https://github.com/heroku/buildpacks-nodejs/pull/1169))
- Dropped provides for `node`, `npm`, `pnpm`, `yarn`, `node_modules`, and `node_build_scripts`. ([#1169](https://github.com/heroku/buildpacks-nodejs/pull/1169))
- Requires `heroku/nodejs` if `package.json`, `index.js`, or `server.js` is detected. ([#1169](https://github.com/heroku/buildpacks-nodejs/pull/1169))

[unreleased]: https://github.com/heroku/buildpacks-nodejs/compare/v5.2.0...HEAD
[5.2.0]: https://github.com/heroku/buildpacks-nodejs/compare/v5.1.4...v5.2.0
[5.1.4]: https://github.com/heroku/buildpacks-nodejs/compare/v5.1.3...v5.1.4
[5.1.3]: https://github.com/heroku/buildpacks-nodejs/compare/v5.1.2...v5.1.3
[5.1.2]: https://github.com/heroku/buildpacks-nodejs/compare/v5.1.1...v5.1.2
[5.1.1]: https://github.com/heroku/buildpacks-nodejs/compare/v5.1.0...v5.1.1
[5.1.0]: https://github.com/heroku/buildpacks-nodejs/compare/v5.0.1...v5.1.0
[5.0.1]: https://github.com/heroku/buildpacks-nodejs/compare/v5.0.0...v5.0.1
[5.0.0]: https://github.com/heroku/buildpacks-nodejs/releases/tag/v5.0.0
