# Yarn Buildpack Implementation Notes

## devDependencies and pruning

The classic buildpack supports three strategies for `devDependencies`: 
1) install devDependencies, then prune them after the build,
2) never install devDependencies,
3) install devDependencies but don't prune them. 

#3 fits almost all apps, but comes at the cost of larger images. We don't have
slug size limits with CNB's, so we don't need the other modes immediately. For
now, we'll punt on #1 and #2.

## Yarn 2+

There are a bunch of changes in yarn 2 (and to a lesser extent, yarn 3). There
are different install flags, a different configuration file (and format), and
different installation modes. Yarn 2+ doesn't have high adoption yet, but this
buildpack aims to support 2, 3, and 4 alongside Yarn 1.

## pnp (Plug'n'Play)

In this mode, `node_modules` is not populated and a `.pnp.cjs` is created
instead. `node_modules` should not between created, copied, symlinked, or 
listed as an app slice in this mode.

## zero-install

Yarn 2+ supports zero install, that is, the customer provides a dependency
cache with the codebase. In this mode, the buildpack does not need to
manage a dependency cache. Yarn itself is bullish on this mode, so this mode
should be well supported.

## node_modules caching

Hypothesis: we shouldn't cache the `node_modules` directly. Yarn 1, 2, 3, and 4
supports a global cache of dependencies. It is likely that the difference 
between restoring a compiled `node_modules` and rebuilding `node_modules` 
from a pristine yarn cache is negligible. In classic yarn, this can be done 
with  `yarn install --cache-folder /some/path/cache`, in yarn 2+, the equivalent
is `yarn config set enableGlobalCache=true globalFolder=/some/path` 
then `yarn install`.

## build scripts

`heroku-postbuild` is weird, because if we see it, we'll run it instead of
the `build` script. But if it's truly `post`, shouldn't it run after, rather
than instead of?

Wouldn't `heroku-build` make more sense as the alternative to `build`? That
would also help `heroku-postbuild` to make more sense, as it would actually
run after either `heroku-build` or `build`.

`heroku-prebuild` actually runs before the build, so it's probably fine as-is. 
But experience tells me not many folks use this. We could wait until someone
asks for it.

`heroku-cleanup` usually runs after module pruning, but this buildpack isn't
pruning yet, so we don't need this one yet.

## corepack

Node.js 16.9+ ships with corepack. Corepack provides shims for `yarn` with
`corepack enable`. Instead of downloading the Yarn CLI, we could use `corepack`
instead. We can check if the `package.json` has a `{ "packageManager":
"yarn@3.1" }` to decide whether to use corepack or download yarn according based
on the `engines.yarn` syntax.

## compile checks

There are a few compile-time checks we should probably fail or warn about:

- FAIL: There is a package-lock.json or npm-shrinkwrap.json. The buildpack can't  
  tell whether the app prefers npm or yarn.
- WARN: There is a .npmrc file. This is a hint that the app is configured to use
  npm.
- WARN: There is a .yarnrc file but yarn > 2 is detected.
- WARN: There is a .yarnrc.yml file but yarn@1 is detected.
- WARN: There is a `packageManager` key in package.json that does not have
  `yarn` in it. App is using corepack, but not using corepack for yarn. It's 
  not clear in this scenario which package manager to use.
