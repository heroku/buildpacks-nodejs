#!/usr/bin/env bash

detect_function_app() {
	local app_dir="${1:?}"
	[[ -f "${app_dir}/function.toml" || -f "${app_dir}/project.toml" ]]
}

write_to_build_plan() {
	local build_plan="${1:?}"
	cat >"${build_plan}" <<-EOF
		[[provides]]
		name = "nodejs-function-runtime"

		[[requires]]
		name = "nodejs-function-runtime"

		[[requires]]
		name = "node"
	EOF
}
