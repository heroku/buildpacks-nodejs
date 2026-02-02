# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

### Changed

- Support pnpm workspace pruning. ([#1270](https://github.com/heroku/buildpacks-nodejs/pull/1270)) 
 
## [5.3.5] - 2026-01-27

### Added

- 25.5.0 (linux-amd64, linux-arm64)

## [5.3.4] - 2026-01-20

### Added

- 25.4.0 (linux-amd64, linux-arm64)

## [5.3.3] - 2026-01-14

### Added

- 25.3.0 (linux-amd64, linux-arm64)
- 24.13.0 (linux-amd64, linux-arm64)
- 22.22.0 (linux-amd64, linux-arm64)
- 20.20.0 (linux-amd64, linux-arm64)

## [5.3.2] - 2025-12-11

### Added

- 24.12.0 (linux-amd64, linux-arm64)

## [5.3.1] - 2025-12-09

### Changed

- Improved download stability by switching from parallel to sequential actions for request, checksum validation, and extraction. ([#1249](https://github.com/heroku/buildpacks-nodejs/pull/1249))

## [5.3.0] - 2025-12-03

### Changed

- Updated default Node.js version to `24.x`. ([#1240](https://github.com/heroku/buildpacks-nodejs/pull/1240))

## [5.2.9] - 2025-11-26

### Added

- 20.19.6 (linux-amd64, linux-arm64)

## [5.2.8] - 2025-11-18

### Added

- 25.2.1 (linux-amd64, linux-arm64)

## [5.2.7] - 2025-11-14

### Changed

- Updated `libcnb` dependencies to `0.30.2` to pull in instrumentation improvements. ([#1234](https://github.com/heroku/buildpacks-nodejs/pull/1234))

## [5.2.6] - 2025-11-13

### Added

- 25.2.0 (linux-amd64, linux-arm64)
- 24.11.1 (linux-amd64, linux-arm64)

## [5.2.5] - 2025-11-10

### Added

- Capture dependency information from `package.json`. ([#1228](https://github.com/heroku/buildpacks-nodejs/pull/1228))
- Error `id` for diagnostics. ([#1230](https://github.com/heroku/buildpacks-nodejs/pull/1230))

### Changed

- Buildpack output and instrumentation now records version ranges in their original form. ([#1229](https://github.com/heroku/buildpacks-nodejs/pull/1229))

## [5.2.4] - 2025-11-07

### Added

- Instrumented buildpack with performance and diagnostic metrics. ([#1216](https://github.com/heroku/buildpacks-nodejs/pull/1216))

## [5.2.3] - 2025-11-04

### Changed

- Use a custom executable wrapper for vendored `yarn` script. ([#1224](https://github.com/heroku/buildpacks-nodejs/pull/1224))

## [5.2.2] - 2025-10-30

### Changed

- Ensure linked binaries are executable. ([#1219](https://github.com/heroku/buildpacks-nodejs/pull/1219))

## [5.2.1] - 2025-10-30

### Added

- 25.1.0 (linux-amd64, linux-arm64)
- 24.11.0 (linux-amd64, linux-arm64)
- 22.21.1 (linux-amd64, linux-arm64)

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

[unreleased]: https://github.com/heroku/buildpacks-nodejs/compare/v5.3.5...HEAD
[5.3.5]: https://github.com/heroku/buildpacks-nodejs/compare/v5.3.4...v5.3.5
[5.3.4]: https://github.com/heroku/buildpacks-nodejs/compare/v5.3.3...v5.3.4
[5.3.3]: https://github.com/heroku/buildpacks-nodejs/compare/v5.3.2...v5.3.3
[5.3.2]: https://github.com/heroku/buildpacks-nodejs/compare/v5.3.1...v5.3.2
[5.3.1]: https://github.com/heroku/buildpacks-nodejs/compare/v5.3.0...v5.3.1
[5.3.0]: https://github.com/heroku/buildpacks-nodejs/compare/v5.2.9...v5.3.0
[5.2.9]: https://github.com/heroku/buildpacks-nodejs/compare/v5.2.8...v5.2.9
[5.2.8]: https://github.com/heroku/buildpacks-nodejs/compare/v5.2.7...v5.2.8
[5.2.7]: https://github.com/heroku/buildpacks-nodejs/compare/v5.2.6...v5.2.7
[5.2.6]: https://github.com/heroku/buildpacks-nodejs/compare/v5.2.5...v5.2.6
[5.2.5]: https://github.com/heroku/buildpacks-nodejs/compare/v5.2.4...v5.2.5
[5.2.4]: https://github.com/heroku/buildpacks-nodejs/compare/v5.2.3...v5.2.4
[5.2.3]: https://github.com/heroku/buildpacks-nodejs/compare/v5.2.2...v5.2.3
[5.2.2]: https://github.com/heroku/buildpacks-nodejs/compare/v5.2.1...v5.2.2
[5.2.1]: https://github.com/heroku/buildpacks-nodejs/compare/v5.2.0...v5.2.1
[5.2.0]: https://github.com/heroku/buildpacks-nodejs/compare/v5.1.4...v5.2.0
[5.1.4]: https://github.com/heroku/buildpacks-nodejs/compare/v5.1.3...v5.1.4
[5.1.3]: https://github.com/heroku/buildpacks-nodejs/compare/v5.1.2...v5.1.3
[5.1.2]: https://github.com/heroku/buildpacks-nodejs/compare/v5.1.1...v5.1.2
[5.1.1]: https://github.com/heroku/buildpacks-nodejs/compare/v5.1.0...v5.1.1
[5.1.0]: https://github.com/heroku/buildpacks-nodejs/compare/v5.0.1...v5.1.0
[5.0.1]: https://github.com/heroku/buildpacks-nodejs/compare/v5.0.0...v5.0.1
[5.0.0]: https://github.com/heroku/buildpacks-nodejs/releases/tag/v5.0.0
