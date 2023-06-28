#!/usr/bin/env bash
set -euo pipefail

# This script is present to support cutlass integration tests which looks for a `build.sh` file and, if found, will
# execute it to produce a compiled buildpack which it then expects to find in the ./target directory.

buildpack_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
cd "$buildpack_dir"
output_dir=$(cargo libcnb package --release)
mv "$output_dir" "${buildpack_dir}/target"
