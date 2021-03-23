#!/usr/bin/env bash

# shellcheck disable=SC2128
bp_dir=$(
	cd "$(dirname "$BASH_SOURCE")" || exit
	cd ..
	pwd
)

# shellcheck source=/dev/null
source "$bp_dir/lib/utils/log.sh"
# shellcheck source=/dev/null
source "$bp_dir/lib/utils/json.sh"

# store_node_version() {
# 	local layers_dir=$1
#
# 	if [[ -f "${layers_dir}/store.toml" ]]; then
# 		local prev_node_version
# 		# shellcheck disable=SC2002
# 		prev_node_version=$(cat "${layers_dir}/store.toml" | grep node_version | xargs | cut -d " " -f3)
#
# 		mkdir -p "${layers_dir}/env"
# 		if [[ -s "${layers_dir}/env/PREV_NODE_VERSION" ]]; then
# 			rm -rf "${layers_dir}/env/PREV_NODE_VERSION"
# 		fi
# 		echo -e "$prev_node_version" >>"${layers_dir}/env/PREV_NODE_VERSION"
# 	fi
# }

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

write_to_store_toml() {
	local layers_dir=$1

	if [[ ! -f "${layers_dir}/store.toml" ]]; then
		touch "${layers_dir}/store.toml"
		cat <<TOML >"${layers_dir}/store.toml"
[metadata]
last_stack = "$CNB_STACK_ID"
TOML
	fi
}

clear_cache_on_node_version_change() {
	local layers_dir=$1
	local curr_node_version
	local prev_node_version
	# shellcheck disable=SC2002
	curr_node_version="$(echo $(node -v))"
	prev_node_version=$(cat "${layers_dir}/env.build/PREV_NODE_VERSION")

	if [[ "$curr_node_version" != "$prev_node_version" ]]; then
		info "Deleting cache because node version changed from \"$prev_node_version\" to \"$curr_node_version\""
		rm -rf "${layers_dir:?}/*"
	fi
}

detect_out_dir() {
	local build_dir=$1

	out_dir=$(json_get_key "$build_dir/tsconfig.json" ".compilerOptions.outDir")

	[[ -f "$build_dir/$out_dir" ]]
}

check_tsc_binary() {
	local build_dir=$1

	[[ -f "$build_dir/node_modules/typescript/bin/tsc" ]]
}
