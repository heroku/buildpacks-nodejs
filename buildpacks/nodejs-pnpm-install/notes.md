Implementation Notes for heroku/nodejs-pnpm

## Installing pnpm

pnpm reccommends installing with corepack. corepack is gaining traction in the
ecosystem as well. We already have a corepack buildpack,
so we can leverage that. Setting `"packageManager": "pnpm@8.1.1"` in
package.json should do the trick. The disadvantage(s) to this method are 1)
corepack doesn't verify the `pnpm` installation and 2) `packageManager`
doesn't support semver constraints.

We could support the `"engines": { "pnpm": "8.x" }` format, since it matches
what we already have for `node` and `yarn`. This type of setting does offer
some protection from running the wrong pnpm version on a project for local dev.
Installing `pnpm` based on this setting is a bit of a heroku-ism. If we did this,
we'd probably want to mirror `pnpm` and keep a local inventory. We could verify
installations, and provide semver constraint resolution. We could probably start 
without support for this method, and add it in later if we decide corepack is 
slow to implement verification.

If we did decide to install from our mirror, should it happen in another
buildpack? Maybe something like `heroku/nodejs-pnpm-engine`? I think yes?

## Installing dependencies

pnpm's distinctive feature is it's module storage system. It's sort of a three
layer system, where the node_modules folder is filled with symlinks to a
"virtual" store folder, which has hard links for known dependencies to a
global/addressable store folder. This leads to a storage-efficient design,
especially in situations where the same module is consumed in multiple projects
or multiple places in the module hierarchy.

A caveat to this approach is that hard links aren't allowed to cross device
boundaries. CNB builds tend to occur in multi-device environments - for instance
/workspace and /layers are different mounts, so we can't hard-link between them.
If pnpm detects cross-device virtual to addressable linking, it will fallback
to copying those files, which of course reduces much of the efficiency of pnpm's
principal feature.

We probably want the global/addressable store to live in /layers, so we can
cache modules between builds. To avoid the copy-fallback mentioned above, it
also means we probably want the virtual store to also live in /layers. The
global, content-addressable store can be build+cache and the virtual store can
be build+launch. This allows the global cache to include a broader list of
packages (those from another branch, or old versions), but not bloat the run
image. The virtual store will only include the packages that are actively
in use.

There is a setting for modules-cache-max-age. I'm not sure how this works yet.
It would be great if the global addressable store was auto-pruned during
`npm install`, so the global addressable store doesn't grow unbounded. If that
is not the case, we'll have to implement something to keep the cache under
control. We could invalidate the entire cache after a number of builds or run
`pnpm store prune`. We probably wouldn't want to run `pnpm store prune` after every build, for the reasons stated [here](https://pnpm.io/cli/store#prune). 

`pnpm prune` also exists, but currently, we rebuild the virtual store on each 
build, so there shouldn't any extra dependencies in there to remove -- that is
unless we were to add support for devDependency or optionalDependency pruning 
(currently, we install dependencies and devDependencies for npm, yarn, and pnpm 
apps with no pruning available).

## Scripts

Like the npm and yarn buildpacks, we should probably run the same lifecycle
hooks - heroku-prebuild, heroku-build, heroku-postbuild. Though we're repeating
the same logic in three buildpacks. The logic is easy-enough to centralize
with a library crate. But it begs the question, would it make more sense to
have a `heroku/nodejs-scripts` buildpack to execute the lifecycle hooks instead?


## Scoping this buildpack

The current implementation of this buildpack assumes that `pnpm` is installed
by another buildpack. The `corepack` buildpack can do that. We could call
this buildpack `heroku/nodejs-pnpm-install`, to hint that it only runs
`pnpm install`. We could leave the installation of `pnpm` to other buildpacks,
like `heroku/nodejs-corepack` and/or `heroku/nodejs-pnpm-engine` (the latter
does not exist, but could), and we could leave other functions, like lifecycle
hooks to other buildpacks (like `heroku/nodejs-scripts`, which also does not yet
exist). If we went this way, we might want to use the same
pattern for `yarn` and `npm`.
