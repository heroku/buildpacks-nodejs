# Node.js Yarn Cloud Native Buildpack

Heroku's official Cloud Native Buildpack for [Yarn](https://yarnpkg.com).

[![CI](https://github.com/heroku/buildpacks-nodejs/actions/workflows/ci.yml/badge.svg)](https://github.com/heroku/buildpacks-nodejs/actions/workflows/ci.yml)

[![Registry](https://img.shields.io/badge/dynamic/json?url=https://registry.buildpacks.io/api/v1/buildpacks/heroku/nodejs-yarn&label=version&query=$.latest.version&color=DF0A6B&logo=data:image/png;base64,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&labelColor=white)](https://registry.buildpacks.io/buildpacks/heroku/nodejs-yarn)


This buildpack relies on and builds on top of the [Node.js Engine Cloud Native Buildpack](https://github.com/heroku/nodejs-engine-buildpack) to add `yarn` functionality to a Node.js app.

## What it does

- Installs the `yarn` CLI (if it's not already installed) and caches it for reuse between builds
- Installs package.json dependencies (including devDependencies) with 
  `yarn install`. Dependencies are cached between builds to provide fast rebuilds.
- Runs `build` scripts from package.json, including `heroku-prebuild`, 
  `heroku-build` (or `build`), and `heroku-postbuild`.
- Sets the default process type as `yarn run start` if it exists.

## Features

### Supported:

- Yarn major versions 1, 2, and 3.
- Yarn pnp (Plug 'n Play) mode for yarn 2+.
- Yarn zero-installs for yarn 2+.

## Unsupported:

- Optional `devDependencies`. `devDependencies` are always installed.
- Pruning `devDependencies`. `devDependencies` are always installed.

## Reference

### Detect

This buildpack's `bin/detect` will only pass if a `yarn.lock` exists in the
project root. This is done to prevent the buildpack from providing indeterminate
and unpredictable dependency trees.

### Environment Variables

#### PATH

`$PATH` will be modified such that `yarn` is available.

### Zero-installs

Yarn zero-installs are supported for yarn 2+. If the buildpack detects that a
populated yarn cache is provided with the source code, this buildpack will
not attempt to cache dependencies and prefer the provided cache. Additionally,
this buildpack will use the `--check-cache` and `--immutable-cache` to ensure
the provided cache is pristine. To use zero-installs, make sure the project's
.gitignore matches [yarn's suggestion](https://yarnpkg.com/getting-started/qa#which-files-should-be-gitignored).

### Plug 'n Play

Yarn plug 'n play is supported for yarn 2+. Ensure `nodeLinker: "pnp"` is in
the project's `.yarnrc.yml` to use this feature.

### Scripts

After dependencies are installed, build scripts will be run in this order: 
`heroku-prebuild`, `heroku-build` (falling back to `build` if `heroku-build`
does not exist), `heroku-postbuild`.

### Process types

If a `start` script is detected in `package.json`, the default process type
for the build will be set to `yarn start`.


### Yarn version selection

By default, this buildpack will install the latest yarn version from the `1.22.x`
line. There are two ways to select a different yarn version:

#### `packageManager`

Use the `heroku/nodejs-corepack` buildpack to install yarn. It will install
yarn according to the `packageManager` key in `package.json`. For example:

```json5
// package.json
{
  "packageManager": "yarn@3.1.2"
}
```

#### `engines.yarn`

Alternatively, define `engines.yarn` using a semver range in `package.json`.
For example:

```json5
// package.json
{
  "engines": {
    "yarn": "3.1.x"
  }
}
```

## Usage

To build an app locally into an OCI Image with this buildpack, use the `pack`
command from Cloud Native Buildpacks using both the
[heroku/nodejs-engine](../nodejs-engine) buildpack and this one:

```
pack build example-app-image --buildpack heroku/nodejs-engine --buildpack heroku/nodejs-yarn --path /some/example-app
```

## Build Plan

### Provides

| Name                 | Description                                                                                                      |
|----------------------|------------------------------------------------------------------------------------------------------------------|
| `yarn`               | Allows other buildpacks that require [Yarn][yarn] tooling to depend on this buildpack.                           |
| `node_modules`       | Allows other buildpacks to depend on the Node modules provided by this buildpack.                                |
| `node_build_scripts` | Allows other buildpacks to depend on the [build script execution](#scripts) behavior provided by this buildpack. |

### Requires

| Name                 | Description                                                                                                                                                                                                               |
|----------------------|---------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|
| `node`               | To execute `pnpm` a [Node.js][Node.js] runtime is required. It can be provided by the [`heroku/nodejs-engine`][heroku/nodejs-engine] buildpack.                                                                           |
| `yarn`               | To install node modules, the [Yarn][yarn] package manager is required. It can be provided by either the [`heroku/nodejs-corepack`][heroku/nodejs-corepack] buildpack or this one.                                         |
| `node_modules`       | This is not a strict requirement of the buildpack. Requiring `node_modules` ensures that this buildpack can be used even when no other buildpack requires `node_modules`.                                                 |
| `node_build_scripts` | This is not a strict requirement of the buildpack. Requiring `node_build_scripts` ensures that this buildpack will perform [build script execution](#scripts) even when no other buildpack requires `node_build_scripts`. |          | 

#### Build Plan Metadata Schemas

##### `node_build_scripts`

* `enabled` ([boolean][toml_type_boolean], optional)

###### Example

```toml
[[requires]]
name = "node_build_scripts"

[requires.metadata]
enabled = false # this will prevent build scripts from running
```

## Additional Info

For development, dependencies, contribution, license and other info, please
refer to the [root README.md](../../README.md).

[Node.js]: https://nodejs.org/

[yarn]: https://yarnpkg.com/

[heroku/nodejs-engine]: ../nodejs-engine/README.md

[heroku/nodejs-corepack]: ../nodejs-corepack/README.md

[toml_type_boolean]: https://toml.io/en/v1.0.0#boolean
