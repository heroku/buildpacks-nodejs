#!/usr/bin/env bash
set -euo pipefail

# Builds a meta buildpack by copying itself to the target directory. Since dependent buildpacks
# might also need to be build before packaging, this script will also look for local buildpack references in
# package.toml, execute their build script if present and modifies the meta-buildpack's package.toml
# (within the target directory) to point to the built version of the of the dependency.

buildpack_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
cd "$buildpack_dir"
output_dir=$(cargo libcnb package --release)
mv "$output_dir" "${buildpack_dir}/target"
