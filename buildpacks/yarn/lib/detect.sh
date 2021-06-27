#!/usr/bin/env bash

detect_yarn_lock() {
	local build_dir=$1
	[[ -f "$build_dir/yarn.lock" ]]
}

write_to_build_plan() {
	local build_plan=$1
	cat <<EOF >"$build_plan"
	[[provides]]
	name = "node_modules"

	[[requires]]
	name = "node_modules"

	[[requires]]
	name = "node"
EOF
}