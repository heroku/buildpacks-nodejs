# Yarn Plug'n'Play data

Plug'n'Play data written by different Yarn versions, used by the unit tests for listing the package
archives an app needs at runtime. Each file was generated with the cache outside the project, like
the buildpack's `yarn_cache` layer, then dev dependencies were pruned with the buildpack's prune
plugin. The Plug'n'Play runtime code that follows the data is omitted.

- `yarn-2.4.1`, `yarn-3.8.7`, `yarn-4.19.0`: a project with production dependencies that cover each
  way Plug'n'Play can locate a package: archives, virtual packages, unplugged packages, workspaces,
  portals, an alias, a package patched by Yarn, and an optional dependency skipped on this platform.
  Yarn 2 names the runtime `.pnp.js`.
- `yarn-4.19.0-no-inlining`: the same project with `pnpEnableInlining: false`, which writes the data
  to `.pnp.data.json`.
- `yarn-4.19.0-unplugged-only`: a project whose only dependency is unplugged, so no archives are
  needed.
- `yarn-4.19.0-no-dependencies`: a project with no dependencies.

Each fixture directory also has an `expected-archives.txt`: the package archives Yarn's own
Plug'n'Play API (`pnpapi`) reports the app loads, one per line, listed by the generator's
`list-pnp-archives.cjs`. The tests compare the buildpack's parsing of the data against it.

## Regenerating

```sh
cargo generate-yarn-fixtures
```

This needs Node.js and Corepack. It runs `crates/xtask-generate-yarn-fixtures`, which installs the projects in that crate's
`projects/` directory with each Yarn version (through Corepack, so it needs network access),
prunes dev dependencies with the buildpack's prune plugin, and rewrites every fixture. Package
versions are pinned, so only changes made by Yarn itself should show up in the diff. The
`@parcel/watcher-win32-x64` optional dependency is only skipped on non-Windows machines.
