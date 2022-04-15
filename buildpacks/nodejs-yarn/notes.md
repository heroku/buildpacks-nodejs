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
buildpack aims to support it alongside Yarn 1.

## pnp (Plug'n'Play)

In this mode, `node_modules` is not populated and a `.pnp.cjs` is created
instead. `node_modules` should not between created, copied, symlinked, or 
listed as an app slice in this mode.

## zero-install

Yarn 2+ supports zero install, that is, the customer provides a dependency
cache with the codebase. In this mode, the buildpack does not need to
manage a dependency cache.

## node_modules caching

Hypothesis: we don't need this. Yarn 1,2, and 3 supports a global cache. It
is likely that the difference between restoring `node_modules` and rebuilding
`node_modules` from a pristine cache is negligible. If the buildpack did decide
to support this functionality, we'd need to instead restore .pnp.js for apps 
that use pnp.

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

## compile checks

There are a few compile-time checks we should probably fail or warn about:

- FAIL: There is a package-lock.json or npm-shrinkwrap.json. The buildpack can't  
  tell whether the app prefers npm or yarn.
- WARN: There is a .npmrc file. This is a hint that the app is configured to use
  npm.
- WARN: There is a .yarnrc file but yarn > 2 is detected.
- WARN: There is a .yarnrc.yml file but yarn@1 is detected.
- WARN: There is a `packageManager` key. This might mean the app uses corepack
  but there is no buildpack support for corepack managed dependency managers.
