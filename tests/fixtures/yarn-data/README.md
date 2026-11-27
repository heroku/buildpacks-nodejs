# Yarn data

Data written by different Yarn versions for Plug'n'Play projects, used by the unit tests.

## Plug'n'Play data

Plug'n'Play data used by the unit tests for listing the package archives an app needs at runtime. Each file was generated with the cache outside the project, like
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

## Yarn mirror data

Yarn mirror and cache folder listings used by the unit tests for removing package archives from the
mirror that an app no longer uses. Each fixture is generated like a rebuild: a build fills the
mirror, the app changes, then the next build restores the mirror and installs into an empty cache
folder, like the buildpack's `yarn_mirror` and `yarn_cache` layers. Every fixture starts from the
same project as `yarn-<version>`.

- `yarn-<version>-dependencies-removed`: `yarn remove resolve dotenv` removes a production
  dependency, whose dependencies include a package patched by Yarn, and a dev dependency.
- `yarn-<version>-cache-key-changed`: `compressionLevel: 9` changes the cache key and the lockfile is
  updated, so every package gets a new mirror archive. A Yarn upgrade that changes the cache key
  leaves the mirror in the same state.

Each fixture directory has:

- `mirror-archives.txt`: the mirror after the rebuild's install.
- `cache-archives.txt`: the cache folder after the rebuild's install.
- `expected-stale-archives.txt`: the mirror archives the app no longer uses. These are worked out by
  Yarn rather than by the buildpack: the changed app is installed into an empty mirror, and every
  archive in `mirror-archives.txt` that install didn't write is stale.
- `yarn.lock`: the changed app's lockfile, whose `cacheKey` is the cache key of the mirror archives
  Yarn uses.

## Regenerating

```sh
cargo generate-yarn-fixtures
```

This needs Node.js and Corepack. It runs `crates/xtask-generate-yarn-fixtures`, which installs the projects in that crate's
`projects/` directory with each Yarn version (through Corepack, so it needs network access),
prunes dev dependencies with the buildpack's prune plugin, generates the mirror fixtures, and
rewrites every fixture. Package
versions are pinned, so only changes made by Yarn itself should show up in the diff. The
`@parcel/watcher-win32-x64` optional dependency is only skipped on non-Windows machines.
