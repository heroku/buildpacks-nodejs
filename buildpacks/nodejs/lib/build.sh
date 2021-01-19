#!/usr/bin/env bash

set -e

# shellcheck disable=SC2128
bp_dir=$(cd "$(dirname "$BASH_SOURCE")"; cd ..; pwd)

# shellcheck source=/dev/null
source "$bp_dir/lib/utils/json.sh"
# shellcheck source=/dev/null
source "$bp_dir/lib/utils/log.sh"
# shellcheck source=/dev/null
source "$bp_dir/lib/utils/toml.sh"

set_up_environment() {
  local layer_dir=$1
  local node_env=${NODE_ENV:-production}

  mkdir -p "${layer_dir}/env.build"

  if [[ ! -s "${layer_dir}/env.build/NODE_ENV.override" ]]; then
    echo -e "$node_env\c" >> "${layer_dir}/env.build/NODE_ENV.override"
  fi
  info "Setting NODE_ENV to ${node_env}"
}

install_or_reuse_toolbox() {
  local layer_dir=$1

  info "Installing toolbox"
  mkdir -p "${layer_dir}/bin"

  if [[ ! -f "${layer_dir}/bin/yj" ]]; then
    info "- yj"
    curl -Ls https://github.com/sclevine/yj/releases/download/v2.0/yj-linux > "${layer_dir}/bin/yj" \
      && chmod +x "${layer_dir}/bin/yj"
  fi

  echo "cache = true" > "${layer_dir}.toml"
  echo "build = true" >> "${layer_dir}.toml"
  echo "launch = false" >> "${layer_dir}.toml"
}

install_or_reuse_node() {
  local layer_dir=$1
  local build_dir=$2

  local engine_node
  local node_version
  local resolved_data
  local node_url
  status "Installing Node"
  info "Getting Node version"
  engine_node=$(json_get_key "$build_dir/package.json" ".engines.node")
  node_version=${engine_node:-12.x}

  info "Resolving Node version"
  resolved_data=$(resolve-version node "$node_version")
  node_url=$(echo "$resolved_data" | cut -f2 -d " ")
  node_version=$(echo "$resolved_data" | cut -f1 -d " ")

  if [[ $node_version == $(toml_get_key_from_metadata "${layer_dir}.toml" "version") ]]; then
    info "Reusing Node v${node_version}"
  else
    info "Downloading and extracting Node v${node_version}"

    mkdir -p "${layer_dir}"
    rm -rf "${layer_dir:?}"/*

    echo "cache = true" > "${layer_dir}.toml"

    {
      echo "build = true"
      echo "launch = true"
      echo -e "[metadata]\nversion = \"$node_version\""
    } >> "${layer_dir}.toml"

    curl -sL "$node_url" | tar xz --strip-components=1 -C "$layer_dir"
  fi
}

parse_package_json_engines() {
  local layer_dir=$1
  local build_dir=$2

  local engine_npm
  local engine_yarn
  local npm_version
  local yarn_version
  local resolved_data
  local yarn_url
  status "Parsing package.json"
  info "Parsing package.json"

  engine_npm=$(json_get_key "$build_dir/package.json" ".engines.npm")
  engine_yarn=$(json_get_key "$build_dir/package.json" ".engines.yarn")

  npm_version=${engine_npm:-6.x}
  yarn_version=${engine_yarn:-1.x}
  resolved_data=$(resolve-version yarn "$yarn_version")
  yarn_url=$(echo "$resolved_data" | cut -f2 -d " ")
  yarn_version=$(echo "$resolved_data" | cut -f1 -d " ")

  cat << TOML > "${layer_dir}.toml"
cache = false
build = true
launch = false

[metadata]
npm_version = "$npm_version"
yarn_url = "$yarn_url"
yarn_version = "$yarn_version"
TOML
}

install_or_reuse_yarn() {
  local layer_dir=$1
  local build_dir=$2

  local engine_yarn
  local yarn_version
  local resolved_data
  local yarn_url

  engine_yarn=$(json_get_key "$build_dir/package.json" ".engines.yarn")
  yarn_version=${engine_yarn:-1.x}
  resolved_data=$(resolve-version yarn "$yarn_version")
  yarn_url=$(echo "$resolved_data" | cut -f2 -d " ")
  yarn_version=$(echo "$resolved_data" | cut -f1 -d " ")
  status "Installing yarn"
  if [[ $yarn_version == $(toml_get_key_from_metadata "${layer_dir}.toml" "yarn_version") ]]; then
    info "Reusing yarn@${yarn_version}"
  else
    info "Installing yarn@${yarn_version}"

    mkdir -p "$layer_dir"
    rm -rf "${layer_dir:?}"/*

    echo "cache = true" > "${layer_dir}.toml"

    {
      echo "build = true"
      echo "launch = true"
      echo -e "[metadata]\nversion = \"$yarn_version\""
    } >> "${layer_dir}.toml"

    curl -sL "$yarn_url" | tar xz --strip-components=1 -C "$layer_dir"
  fi
}

set_node_env() {
  local layer_dir=$1
  local node_env=${NODE_ENV:-production}

  mkdir -p "${layer_dir}/env.launch"
  if [[ ! -s "${layer_dir}/env.launch/NODE_ENV.override" ]]; then
    echo -e "$node_env\c" >> "${layer_dir}/env.launch/NODE_ENV.override"
  fi
}

copy_profile() {
  local layer_dir=$1
  local bp_dir=$2

  mkdir -p "${layer_dir}/profile.d"
  cp "$bp_dir/profile/WEB_CONCURRENCY.sh" "$layer_dir/profile.d"
}

write_launch_toml() {
  local build_dir=$1
  local launch_toml=$2

  local command

  if [[ -f "$build_dir/server.js" ]]; then
    command="node server.js"
  fi

  if [[ -f "$build_dir/index.js" ]]; then
    command="node index.js"
  fi

  if [[ ! "$command" ]]; then
    info "No file to start server"
    info "either use 'docker run' to start container or add index.js or server.js"
  else
    cat <<TOML > "$launch_toml"
[[processes]]
type = "web"
command = "$command"
TOML
  fi
}
