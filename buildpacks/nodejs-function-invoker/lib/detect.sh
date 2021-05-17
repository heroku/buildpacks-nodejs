#!/usr/bin/env bash

detect_function_app() {
	local app_dir="${1:?}"
	[[ -f "${app_dir}/function.toml" || (-f "${app_dir}/project.toml" && $(yj -t <"${app_dir}/project.toml" | jq -r .com.salesforce.type) == "function") ]]
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
