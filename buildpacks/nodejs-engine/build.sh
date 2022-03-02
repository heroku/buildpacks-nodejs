#!/usr/bin/env bash
set -euo pipefail

buildpack_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"

pushd "${buildpack_dir}"

cargo libcnb package --release

rm -rf target
cp -R ../../target/buildpack/debug/heroku_nodejs-engine target
cp package.toml target/

popd
