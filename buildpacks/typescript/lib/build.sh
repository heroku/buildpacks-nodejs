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

	curr_node_version="$(node -v)"
	curr_node_version=${curr_node_version:1} #to truncate the "v" that is concatedated to version in node -v
	if [[ -n "${PREV_NODE_VERSION:-}" ]]; then
		if [[ "$curr_node_version" != "$PREV_NODE_VERSION" ]]; then
			info "Deleting cache because node version changed from \"$PREV_NODE_VERSION\" to \"$curr_node_version\""
			rm -rf "${layers_dir:?}"/*
		fi
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
