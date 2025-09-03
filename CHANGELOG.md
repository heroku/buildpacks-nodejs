# Changelog

All notable changes to this project will be documented in this file.

The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.1.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]

## [5.0.0] - 2025-09-03

### Changed

- Merged functionality from individual buildpacks into `heroku/nodejs`, previous `CHANGELOG.md` files are linked below:
  - [heroku/nodejs-corepack](https://github.com/heroku/buildpacks-nodejs/blob/v4.1.4/buildpacks/nodejs-corepack/CHANGELOG.md)
  - [heroku/nodejs-engine](https://github.com/heroku/buildpacks-nodejs/blob/v4.1.4/buildpacks/nodejs-engine/CHANGELOG.md)
  - [heroku/nodejs-npm-engine](https://github.com/heroku/buildpacks-nodejs/blob/v4.1.4/buildpacks/nodejs-npm-engine/CHANGELOG.md)
  - [heroku/nodejs-npm-install](https://github.com/heroku/buildpacks-nodejs/blob/v4.1.4/buildpacks/nodejs-npm-install/CHANGELOG.md)
  - [heroku/nodejs-pnpm-engine](https://github.com/heroku/buildpacks-nodejs/blob/v4.1.4/buildpacks/nodejs-pnpm-engine/CHANGELOG.md)
  - [heroku/nodejs-pnpm-install](https://github.com/heroku/buildpacks-nodejs/blob/v4.1.4/buildpacks/nodejs-pnpm-install/CHANGELOG.md)
  - [heroku/nodejs-yarn](https://github.com/heroku/buildpacks-nodejs/blob/v4.1.4/buildpacks/nodejs-yarn/CHANGELOG.md)
- Updated provides to `heroku/nodejs`. 
- Dropped provides for `node`, `npm`, `pnpm`, `yarn`, `node_modules`, and `node_build_scripts`.
- Requires `heroku/nodejs` if `package.json`, `index.js`, or `server.js` is detected.

[unreleased]: https://github.com/heroku/buildpacks-nodejs/compare/v5.0.0...HEAD
[5.0.0]: https://github.com/heroku/buildpacks-nodejs/releases/tag/v5.0.0
