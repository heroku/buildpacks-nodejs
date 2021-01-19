# Node.js Cloud Native Buildpack

Cloud Native Buildpacks are buildpacks that turn source code into OCI images. They follow a 4-step process (detect, analyze, build, and export) that outputs an image. The spec can be read about in detail [here](https://github.com/buildpack/spec/blob/main/buildpack.md).

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
git clone git@github.com:heroku/nodejs-engine-buildpack.git
```

### Build the image

Using pack, you're ready to create an image from the buildpack and source code. You will need to add flags that point to the path of the buildpack (`--buildpack`) and the path of the source code (`--path`).

```sh
cd nodejs-engine-buildpack
pack build TEST_IMAGE_NAME --buildpack ../nodejs-engine-buildpack --path ../TEST_REPO_PATH
```

### Local development

The buildpack uses a Golang binary to parse the engine versions from the `package.json`. It's better to create the binaries once locally, so they don't have to be downloaded and rebuilt with every build.

```
make build
```

This builds the binaries specific for the Docker image. The binaries are in the `.gitignore`, so they won't be committed or ever exist in the remote source code.

If you need them for a MacOS, run:

```
make build-local
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

**Binary Tests**

```sh
make binary-tests
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
3. Move the changes from `main` to a new header with the version and date (ie. `1.14.2 (2020-02-30)`) in the `CHANGELOG.md`.
4. Open a pull request.
5. Let the tests run on CI. When tests pass and PR is approved, the branch is ready to be merged.
6. Merge branch to `main` on GitHub.
7. Pull down `main` to local machine.
8. Tag the current `main` with the version. (`git tag v1.14.2`)
9. Push up to GitHub. (`git push origin main --tags`) CI will run the suite and create a new release on successful run.

## Glossary

- buildpacks: provide framework and a runtime for source code. Read more [here](https://buildpacks.io).
- OCI image: [OCI (Open Container Initiative)](https://www.opencontainers.org/) is a project to create open sourced standards for OS-level virtualization, most importantly in Linux containers.
