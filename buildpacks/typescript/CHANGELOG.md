# Changelog
The format is based on [Keep a Changelog](https://keepachangelog.com/en/1.0.0/),
and this project adheres to [Semantic Versioning](https://semver.org/spec/v2.0.0.html).

## [Unreleased]
- Flush cache when stack image changes ([#28](https://github.com/heroku/buildpacks-node/pull/28))
- Trim whitespace when getting stack name ([#29](https://github.com/heroku/buildpacks-node/pull/29))

## [0.2.1] 2021/02/23
- Add license to buildpack.toml ([#17](https://github.com/heroku/buildpacks-node/pull/17))

## [0.2.0]
### Added
- Add heroku-20 support and stack-specific tests ([#13](https://github.com/heroku/nodejs-typescript-buildpack/pull/13))

## [0.1.0]
### Added
- detect if custom tsconfig env var is set ([#10](https://github.com/heroku/nodejs-typescript-buildpack/pull/10))
- change master to main ([#11](https://github.com/heroku/nodejs-typescript-buildpack/pull/11))

## [0.0.2]
### Added
- check if tsc binary is present in the build ([#6](https://github.com/heroku/nodejs-typescript-buildpack/pull/6))
- add typescript binary to PATH ([#7](https://github.com/heroku/nodejs-typescript-buildpack/pull/7))

## [0.0.1]
### Added
- add the $layers_dir argument to bin/build ([#3](https://github.com/heroku/nodejs-typescript-buildpack/pull/3))
- detect outDir directory from tsconfig.json ([#4](https://github.com/heroku/nodejs-typescript-buildpack/pull/4))
