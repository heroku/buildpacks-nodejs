# Node.js NPM Cloud Native Buildpack

This buildpack builds on top of the existing [Node.js Engine Cloud Native Buildpack](https://github.com/heroku/nodejs-engine-buildpack). It runs subsequent scripts after Node is install.

- Run automatically
  - `npm install` or `npm ci`
- Run when configured in `package.json`
  - `npm run build` or `npm run heroku-postbuild`

## Usage

### Install pack

Using `brew` (assuming development is done on MacOS), install `pack`.

```sh
brew tap buildpack/tap
brew install pack
```

If you're using Windows or Linux, follow instructions [here](https://buildpacks.io/docs/install-pack/).

### Install shpec (optional)

This buildpack uses `shpec` for unit tests, so to run them locally, you'll need to install the package.

```sh
curl -sLo- http://get.bpkg.sh | bash
bpkg install rylnd/shpec
```

### Clone the buildpack

Right now, we are prototyping with a local version of the buildpack. Clone it to your machine.

```sh
git clone git@github.com:heroku/nodejs-npm-buildpack.git
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
cd nodejs-npm-buildpack
pack build TEST_IMAGE_NAME --path ../TEST_REPO_PATH --buildpack ../nodejs-engine-buildpack --buildpack ../nodejs-npm-buildpack
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
  id = "heroku/nodejs-engine"
  uri = "../nodejs-engine-buildpack"

[[buildpacks]]
  id = "heroku/nodejs-npm"
  uri = "../nodejs-npm-buildpack"

[[order]]
  group = [
    { id = "heroku/nodejs-engine", version = "0.4.3" },
    { id = "heroku/nodejs-npm", version = "0.1.4" }
  ]

[stack]
  id = "heroku-22"
  build-image = "heroku/heroku:22-cnb-build"
  run-image = "heroku/heroku:22-cnb"
```

Create the builder with `pack`:

```sh
pack create-builder nodejs --config ../heroku-nodejs-builder/builder.toml
```

Now you can use the builder image instead of chaining the buildpacks.

```sh
pack build TEST_IMAGE_NAME --path ../TEST_REPO_PATH --builder nodejs
```

### Common Issues

#### `jq: Permission denied` on a build

This issue may happen if a binary that is installed is not executable. This may happen on a Linux machine or while using a private network, such as a VPN, when using a local buildpack tool. If using `sfdx evergreen` or `pack`, pass in `--network host` to the command.

An example of this command running from the source code directory with a local builder image called `nodejs` would look like this:

```sh
pack build TEST_IMAGE_NAME --builder nodejs --network host
```

If building a function with `sfdx`, a command looks like this:

```sh
sfdx evergreen:functions:build image-repo/myfunction:dev --network host
```

## Testing

The complete test suite needs Docker to run. Make sure to [install Docker first](https://hub.docker.com/search?type=edition&offering=community).

```sh
make test
```

If you want to run individual test suites, that's available too.

**Unit Tests**

To run the tests on the local host, [make sure `shpec` is installed](#install-shpec-optional).

```sh
make unit-test
```

### Unit tests in Docker

Running the `shpec` aren't ideal since the test scripts read and write to the local buildpack directory, so Docker may be preferred.

As suggested above, install [Docker](#testing). Next, run the tests with the Make script:

```sh
make docker-unit-test
```

### Debugging tests

To debug, make changes from the code and rerun with the make command. To see what is happening, I suggest wrapping code blocks in question with `set -x`/`set +x`. It would look like this in the shpec file:

```sh
set -x
it "creates a toolbox.toml"
  install_or_reuse_toolbox "$layers_dir/toolbox"

  assert file_present "$layers_dir/toolbox.toml"
end
set +x
```

## Contributing

1. Open a pull request.
2. Make update to `CHANGELOG.md` under `main` with a description (PR title is fine) of the change, the PR number and link to PR.
3. Let the tests run on CI. When tests pass and PR is approved, the branch is ready to be merged.
4. Merge branch to `main`.

### Release

Note: if you're not a contributor to this project, a contributor will have to make the release for you.

1. Create a new branch (ie. `1.14.2-release`).
2. Update the version in the `buildpack.toml`.
3. Move the changes from `main` to a new header with the version and date (ie. `1.14.2 (2020-02-30)`).
4. Open a pull request.
5. Let the tests run on CI. When tests pass and PR is approved, the branch is ready to be merged.
6. Merge branch to `main`.
7. Pull down `main` to local machine.
8. Tag the current `main` with the version. (`git tag v1.14.2`)
9. Push up to GitHub. (`git push origin main --tags`) CI will run the suite and create a new release on successful run.

## Glossary

- buildpacks: provide framework and a runtime for source code. Read more [here](https://buildpacks.io).
- OCI image: [OCI (Open Container Initiative)](https://www.opencontainers.org/) is a project to create open sourced standards for OS-level virtualization, most importantly in Linux containers.
