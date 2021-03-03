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
source "$bp_dir/lib/utils/log.sh"

clear_cache_on_stack_change() {
	local layers_dir=$1

	if [[ -f "${layers_dir}/store.toml" ]]; then
		local last_stack
		# shellcheck disable=SC2002
		last_stack=$(cat "${layers_dir}/store.toml" | grep last_stack | cut -d " " -f3)

		if [[ "\"$CNB_STACK_ID\"" != "$last_stack" ]]; then
			info "Deleting cache because stack changed from $last_stack to \"$CNB_STACK_ID\""
			rm -rf "${layers_dir:?}"/*
		fi
	fi

	if [[ ! -f "${layers_dir}/store.toml" ]]; then
		touch "${layers_dir}/store.toml"
		cat <<TOML >"${layers_dir}/store.toml"
[metadata]
last_stack = "$CNB_STACK_ID"
TOML
	fi
}

detect_package_lock() {
	local build_dir=$1

	[[ -f "$build_dir/package-lock.json" ]]
}

use_npm_ci() {
	local npm_version
	local major
	local minor

	npm_version=$(npm -v)
	major=$(echo "$npm_version" | cut -f1 -d ".")
	minor=$(echo "$npm_version" | cut -f2 -d ".")

	[[ "$major" -gt "5" || ("$major" == "5" && "$minor" -gt "6") ]]
}

install_or_reuse_npm() {
	local build_dir=$1
	local layer_dir=$2
	local npm_version
	local engine_npm
	local latest_npm_version

	npm_version=$(npm -v)
	engine_npm=$(json_get_key "$build_dir/package.json" ".engines.npm")

	# if no engine version specified
	if [[ -z "$engine_npm" ]]; then
		info "Using npm v${npm_version} from Node"
		return 0
	fi

	# if engine version equals the installed version
	# from either Node or the cache
	if [[ "$engine_npm" == "$npm_version" ]]; then
		info "Using npm v${npm_version} from package.json"
	else
		latest_npm_version=$(npm view npm@"$engine_npm" version | tail -n 1 | cut -d "'" -f2)

		# if installed version is the latest version
		if [[ "$npm_version" == "$latest_npm_version" ]]; then
			info "Using npm v${npm_version} from package.json"
		else
			info "Installing npm v${engine_npm} from package.json"
			mkdir -p "$(npm root -g --prefix "$layer_dir")"
			npm install -g "npm@${engine_npm}" --prefix "$layer_dir" --quiet

			cat <<TOML >"${layer_dir}.toml"
cache = true
build = true
launch = true
TOML
		fi
	fi
}

run_prebuild() {
	local build_dir=$1
	local heroku_prebuild_script

	heroku_prebuild_script=$(json_get_key "$build_dir/package.json" ".scripts[\"heroku-prebuild\"]")

	if [[ $heroku_prebuild_script ]]; then
		npm run heroku-prebuild
	fi
}

install_modules() {
	local build_dir=$1
	local layer_dir=$2

	if detect_package_lock "$build_dir"; then
		info "Installing node modules from ./package-lock.json"
		if use_npm_ci; then
			npm ci
		else
			npm install
		fi
	else
		info "Installing node modules"
		npm install --no-package-lock
	fi
}

install_or_reuse_node_modules() {
	local build_dir=$1
	local layer_dir=$2
	local local_lock_checksum
	local cached_lock_checksum

	if ! detect_package_lock "$build_dir"; then
		install_modules "$build_dir" "$layer_dir"
		return 0
	fi

	touch "$layer_dir.toml"
	mkdir -p "${layer_dir}"

	local_lock_checksum=$(sha256sum "$build_dir/package-lock.json" | cut -d " " -f 1)
	cached_lock_checksum=$(yj -t <"${layer_dir}.toml" | jq -r ".metadata.package_lock_checksum")

	if [[ "$local_lock_checksum" == "$cached_lock_checksum" ]]; then
		info "Reusing node modules"
		cp -r "$layer_dir" "$build_dir/node_modules"
	else
		echo "cache = true" >"${layer_dir}.toml"

		{
			echo "build = false"
			echo "launch = false"
			echo -e "[metadata]\npackage_lock_checksum = \"$local_lock_checksum\""
		} >>"${layer_dir}.toml"

		install_modules "$build_dir" "$layer_dir"

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
		npm run heroku-postbuild
	elif [[ $build_script ]]; then
		npm run build
	fi
}

write_launch_toml() {
	local package_json=$1
	local launch_toml=$2

	if [ "null" != "$(jq -r .scripts.start <"$package_json")" ]; then
		cat <<TOML >"$launch_toml"
[[processes]]
type = "web"
command = "npm start"
TOML
	fi

}

prune_devdependencies() {
	local build_dir=$1
	local npm_version

	npm_version=$(npm -v)

	if [ "$NODE_ENV" != "production" ]; then
		warning "Skip pruning because NODE_ENV is not 'production'."
	else
		npm prune --userconfig "$build_dir/.npmrc" 2>&1
		info "Successfully pruned devdependencies!"
	fi
}

warn_prebuilt_modules() {
	local build_dir=$1
	if [ -e "$build_dir/node_modules" ]; then
		info "node_modules checked into source control" "https://devcenter.heroku.com/articles/node-best-practices#only-git-the-important-bits"
	fi
}
