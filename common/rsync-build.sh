#!/usr/bin/env bash
set -euo pipefail

# Copies the whole buildpack to the target directory while following symlinks.
# Resolving symlinks to regular files is the main purpose of this "build" script.

buildpack_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
target_dir_name="target"
target_dir="${buildpack_dir}/${target_dir_name}"

mkdir "${target_dir}"
rsync -a -L "${buildpack_dir}/" "${target_dir}" --exclude "${target_dir_name}"
