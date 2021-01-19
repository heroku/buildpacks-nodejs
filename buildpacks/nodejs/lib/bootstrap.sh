#!/usr/bin/env bash

set -eo pipefail

# shellcheck disable=SC2128
bp_dir=$(cd "$(dirname "$BASH_SOURCE")"; cd ..; pwd)

# shellcheck source=/dev/null
source "$bp_dir/lib/utils/log.sh"

install_go() {
  local go_dir="${1:?}"
  local go_tgz

  go_tgz="$(mktemp /tmp/go.tgz.XXXXXX)"

  curl --retry 3 -sf -o "$go_tgz" -L https://dl.google.com/go/go1.14.1.linux-amd64.tar.gz
  tar -C "$go_dir" -xzf "$go_tgz"
}

build_cmd() {
  local cmd=$1
  local layer_dir=$2

  go get -d "./cmd/${cmd}/..."
  go build -o "$layer_dir/bin/$cmd" "./cmd/${cmd}/..."
  chmod +x "$layer_dir/bin/$cmd"
}

create_bootstrap_layer() {
  mkdir -p "${layer_dir}"

  echo "cache = true" > "${layer_dir}.toml"
  echo "build = true" >> "${layer_dir}.toml"
  echo "launch = false" >> "${layer_dir}.toml"
}

bootstrap_buildpack() {
  local layer_dir="$1"
  local go_dir

  if [[ -f "$bp_dir/bin/resolve-version" ]]; then
    export PATH=$bp_dir/bin:$PATH
  else
    if [[ -f "$layer_dir/bin/resolve-version" ]]; then
      info "Using previously bootstrapped binaries"
    else
      info "Bootstrapping buildpack"

      go_dir="$(mktemp -d)"
      install_go "$go_dir"
      export PATH="$PATH:${go_dir}/go/bin"

      cd "$bp_dir"

      create_bootstrap_layer "$layer_dir"
      build_cmd "resolve-version" "$layer_dir"
    fi
    export PATH=$layer_dir/bin:$PATH
  fi
}
