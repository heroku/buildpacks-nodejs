# Node.js Yarn Cloud Native Buildpack

This buildpack builds on top of the existing [Node.js Engine Cloud Native Buildpack](https://github.com/heroku/nodejs-engine-buildpack). It runs subsequent scripts after Node is install.

- Run automatically
  - `yarn install`
- Run when configured in `package.json`
  - `yarn build` or `yarn heroku-postbuild`

## Usage

### Install pack

Using `brew` (assuming development is done on MacOS), install `pack`.

```sh
brew tap buildpack/tap
brew install pack
```

If you're using Windows or Linux, follow instructions [here](https://buildpacks.io/docs/install-pack/).

### Clone the buildpack

Right now, we are prototyping with a local version of the buildpack. Clone it to your machine.

```sh
git clone git@github.com:heroku/nodejs-yarn-buildpack.git
```

Clone the Heroku Node.js Engine Cloud Native Buildpack.

```sh
cd .. # change from nodejs-npm-buildpack directory
git clone git@github.com:heroku/nodejs-engine-buildpack.git
```

### Build the image

#### with buildpacks

Using pack, you're ready to create an image from the buildpack and source code. You will need to add flags that point to the path of the source code (`--path`) and the paths of the buildpacks (`--buildpack`).

```sh
cd nodejs-yarn-buildpack
pack build TEST_IMAGE_NAME --path ../TEST_REPO_PATH --buildpack ../nodejs-engine-buildpack --buildpack ../nodejs-yarn-buildpack
```

#### with a builder

You can also create a `builder.toml` file that will have explicit directions when creating a buildpack. This is useful when there are multiple "detect" paths a build can take (ie. yarn vs. npm commands).

In a directory outside of this buildpack, create a builder file:

```sh
cd ..
mkdir heroku_nodejs_builder
touch heroku_nodejs_builder/builder.toml
```

For local development, you'll want the file to look like this:

```toml
[[buildpacks]]
  id = "heroku/nodejs-engine-buildpack"
  uri = "../nodejs-engine-buildpack"

[[buildpacks]]
  id = "heroku/nodejs-yarn-buildpack"
  uri = "../nodejs-yarn-buildpack"

[[order]]
  group = [
    { id = "heroku/nodejs-engine-buildpack", version = "0.0.1" },
    { id = "heroku/nodejs-yarn-buildpack", version = "0.0.1" }
  ]

[stack]
  id = "heroku-18"
  build-image = "heroku/pack:18"
  run-image = "heroku/pack:18"
```

Create the builder with `pack`:

```sh
pack create-builder nodejs --builder-config ../heroku-nodejs-builder/builder.toml
```

Now you can use the builder image instead of chaining the buildpacks.

```sh
pack build TEST_IMAGE_NAME --path ../TEST_REPO_PATH --builder nodejs
```

## Contributing

1. Open a pull request.
2. Make update to `CHANGELOG.md` under `master` with a description (PR title is fine) of the change, the PR number and link to PR.
3. Let the tests run on CI. When tests pass and PR is approved, the branch is ready to be merged.
4. Merge branch to `master`.

### Release

Note: if you're not a contributor to this project, a contributor will have to make the release for you.

1. Create a new branch (ie. `1.14.2-release`).
2. Update the version in the `buildpack.toml`.
3. Move the changes from `master` to a new header with the version and date (ie. `1.14.2 (2020-02-30)`).
4. Open a pull request.
5. Let the tests run on CI. When tests pass and PR is approved, the branch is ready to be merged.
6. Merge branch to `master`.
7. Pull down `master` to local machine.
8. Tag the current `master` with the version. (`git tag v1.14.2`)
9. Push up to GitHub. (`git push origin master --tags`) CI will run the suite and create a new release on successful run.

## Glossary

- buildpacks: provide framework and a runtime for source code. Read more [here](https://buildpacks.io).
- OCI image: [OCI (Open Container Initiative)](https://www.opencontainers.org/) is a project to create open sourced standards for OS-level virtualization, most importantly in Linux containers.
