#!/usr/bin/env bash

write_to_build_plan() {
	local build_plan=$1
	cat <<EOF >"$build_plan"
	[[provides]]
	name = "node"

	[[requires]]
	name = "node"
EOF
}
