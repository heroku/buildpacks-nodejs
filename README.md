# Heroku Cloud Native Node.js Buildpacks
[![CircleCI](https://circleci.com/gh/heroku/buildpacks-node/tree/main.svg?style=shield)](https://circleci.com/gh/heroku/buildpacks-node/tree/main)

Heroku's official [Cloud Native Buildpacks](https://buildpacks.io) for the Node.js ecosystem.

## Included Buildpacks
### Languages
Language buildpacks are meta-buildpacks that aggregate other buildpacks for convenient use. Use these if you want
to build your application.

- `heroku/nodejs` ([Readme](meta-buildpacks/nodejs/README.md), [Changelog](meta-buildpacks/nodejs/CHANGELOG.md))

### Misc

- `heroku/nodejs-engine` ([Readme](buildpacks/nodejs/README.md), [Changelog](buildpacks/nodejs/CHANGELOG.md))
- `heroku/nodejs-npm` ([Readme](buildpacks/npm/README.md), [Changelog](buildpacks/npm/CHANGELOG.md))
- `heroku/nodejs-typescript` ([Readme](buildpacks/typescript/README.md), [Changelog](buildpacks/typescript/CHANGELOG.md))
- `heroku/nodejs-yarn` ([Readme](buildpacks/yarn/README.md), [Changelog](buildpacks/yarn/CHANGELOG.md))

## Classic Heroku Buildpacks

If you're looking for the repositories of the classic Node.js Heroku buildpacks than can be used on the Heroku platform,
use the links below for your convenience.

- [heroku/nodejs](https://github.com/heroku/heroku-buildpack-nodejs)

## Building
Many of the buildpacks in this repository require a separate build step before they can be used. By convention, build
scripts must be located in a file named `build.sh` in the buildpack root directory.

### Build script conventions
`build.sh` scripts:
- **MUST NOT** depend on a specific working directory and can be called from anywhere
- **MUST** write the finished buildpack to `target/` within the buildpack directory

### Dependencies
- [Bash](https://www.gnu.org/software/bash/) >= `5.0`
- [yj](https://github.com/sclevine/yj) >= `5.0.0` in `$PATH`
- [jq](https://github.com/stedolan/jq) >= `1.6` in `$PATH`

## License
Licensed under the MIT License. See [LICENSE](./LICENSE) file.
