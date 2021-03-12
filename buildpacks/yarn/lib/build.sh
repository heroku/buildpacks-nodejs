#!/usr/bin/env bash

set -e

# shellcheck disable=SC2128
bp_dir=$(
	cd "$(dirname "$BASH_SOURCE")"
	cd ..
	pwd
)

# shellcheck source=/dev/null
source "$bp_dir/lib/utils/env.sh"
# shellcheck source=/dev/null
source "$bp_dir/lib/utils/json.sh"
# shellcheck source=/dev/null
source "$bp_dir/lib/detect.sh"

fail_multiple_lockfiles() {
	local build_dir=$1
	local has_yarn_lockfile=false
	if [[ -f "$build_dir/yarn.lock" ]]; then
		has_yarn_lockfile=true
	fi
	if $has_yarn_lockfile && [[ -f "$build_dir/package-lock.json" || -f "$build_dir/npm-shrinkwrap.json" ]]; then
		error "Build failed because two different lockfiles were detected: package-lock.json and yarn.lock"
		exit 1
	fi
}

store_node_version() {
	local layers_dir=$1

	if [[ -f "${layers_dir}/store.toml" ]]; then
		local prev_node_version
		# shellcheck disable=SC2002
		prev_node_version=$(cat "${layers_dir}/store.toml" | grep node_version | xargs | cut -d " " -f3)

		mkdir -p "${layers_dir}/env"
		if [[ -s "${layers_dir}/env/PREV_NODE_VERSION" ]]; then
			rm -rf "${layers_dir}/env/PREV_NODE_VERSION"
		fi
		echo -e "$prev_node_version" >>"${layers_dir}/env/PREV_NODE_VERSION"
	fi
}

clear_cache_on_stack_change() {
	local layers_dir=$1

	if [[ -f "${layers_dir}/store.toml" ]]; then
		local last_stack
		# shellcheck disable=SC2002
		last_stack=$(cat "${layers_dir}/store.toml" | grep last_stack | xargs | cut -d " " -f3)

		if [[ "$CNB_STACK_ID" != "$last_stack" ]]; then
			info "Deleting cache because stack changed from \"$last_stack\" to \"$CNB_STACK_ID\""
			rm -rf "${layers_dir:?}"/*
		fi
	fi

	if [[ ! -f "${layers_dir}/store.toml" ]]; then
		curr_node_version="$(echo $(node -v))"
		touch "${layers_dir}/store.toml"
		cat <<TOML >"${layers_dir}/store.toml"
[metadata]
last_stack = "$CNB_STACK_ID"
node_version = "$curr_node_version"
TOML
	fi
}

run_prebuild() {
	local build_dir=$1
	local heroku_prebuild_script

	heroku_prebuild_script=$(json_get_key "$build_dir/package.json" ".scripts[\"heroku-prebuild\"]")

	if [[ $heroku_prebuild_script ]]; then
		yarn heroku-prebuild
	fi
}

install_modules() {
	local build_dir=$1

	if detect_yarn_lock "$build_dir"; then
		echo "---> Installing node modules from ./yarn.lock"
		yarn install
	else
		echo "---> Installing node modules"
		yarn install --no-lockfile
	fi
}

clear_cache_on_node_version_change() {
	local layers_dir=$1

	if [[ -f "${layers_dir}/store.toml" ]]; then
		local curr_node_version
		local prev_node_version
		# shellcheck disable=SC2002
		curr_node_version="$(echo $(node -v))"
		prev_node_version=$(cat "${layers_dir}/env/PREV_NODE_VERSION")

		if [[ "$curr_node_version" != "$prev_node_version" ]]; then
			info "Deleting cache because node version changed from \"$prev_node_version\" to \"$curr_node_version\""
			rm -rf "${layers_dir:?}/node_modules"
			rm -rf "${layers_dir:?}/store.toml"
		fi
	fi

	if [[ ! -f "${layers_dir}/store.toml" ]]; then
		curr_node_version="$(echo $(node -v))"
		touch "${layers_dir}/store.toml"
		cat <<TOML >"${layers_dir}/store.toml"
[metadata]
last_stack = "$CNB_STACK_ID"
node_version = "$curr_node_version"
TOML
	fi
}

install_or_reuse_node_modules() {
	local build_dir=$1
	local layer_dir=$2
	local local_lock_checksum
	local cached_lock_checksum

	touch "$layer_dir.toml"
	mkdir -p "${layer_dir}"
	cp -r "$layer_dir" "$build_dir/node_modules"

	local_lock_checksum=$(sha256sum "$build_dir/yarn.lock" | cut -d " " -f 1)
	cached_lock_checksum=$(yj -t <"${layer_dir}.toml" | jq -r ".metadata.yarn_lock_checksum")

	if [[ "$local_lock_checksum" == "$cached_lock_checksum" ]]; then
		echo "---> Reusing node modules"
	else
		echo "cache = true" >"${layer_dir}.toml"

		{
			echo "build = false"
			echo "launch = false"
			echo -e "[metadata]\nyarn_lock_checksum = \"$local_lock_checksum\""
		} >>"${layer_dir}.toml"

		install_modules "$build_dir"

		if [[ -d "$build_dir/node_modules" && -n "$(ls -A "$build_dir/node_modules")" ]]; then
			cp -r "$build_dir/node_modules/." "$layer_dir"
		fi
	fi
}

run_build() {
	local build_dir=$1
	local build_script
	local heroku_postbuild_script

	build_script=$(json_get_key "$build_dir/package.json" ".scripts.build")
	heroku_postbuild_script=$(json_get_key "$build_dir/package.json" ".scripts[\"heroku-postbuild\"]")

	if [[ $heroku_postbuild_script ]]; then
		yarn heroku-postbuild
	elif [[ $build_script ]]; then
		yarn build
	fi
}

write_launch_toml() {
	local package_json=$1
	local launch_toml=$2

	if [ "null" != "$(jq -r .scripts.start <"$package_json")" ]; then
		cat <<TOML >"$launch_toml"
[[processes]]
type = "web"
command = "yarn start"
TOML
	fi

}
