#!/usr/bin/env bash
# shellcheck source-path=SCRIPTDIR/..

source "${CNB_BUILDPACK_DIR}/lib/failures.sh"
source "${CNB_BUILDPACK_DIR}/lib/utils/download.sh"
source "${CNB_BUILDPACK_DIR}/lib/utils/json.sh"
source "${CNB_BUILDPACK_DIR}/lib/utils/log.sh"
source "${CNB_BUILDPACK_DIR}/lib/utils/toml.sh"

run_initial_checks() {
	build_dir="$1"
	fail_on_no_main_key "$build_dir"
	fail_on_no_main_file "$build_dir"
}

install_runtime() {
	local layers_dir="${1:?}"
	local runtime_layer_dir="${layers_dir}/sf-fx-runtime-nodejs"
	local runtime_layer_toml="${layers_dir}/sf-fx-runtime-nodejs.toml"

	status "Installing Node.js function runtime"

	mkdir -p "${runtime_layer_dir}"
	cat >"${runtime_layer_toml}" <<-EOF
		[types]
		launch = true
		build = false
		cache = false
	EOF

	local runtime_package
	runtime_package=$(toml_get_key_from_metadata "${CNB_BUILDPACK_DIR}/buildpack.toml" "runtime.package")

	info "Installing ${runtime_package}..."

	npm install -g --prefix "${runtime_layer_dir}" "${runtime_package}"

	info "${runtime_package} installation successful"
}

install_scripts() {
	local layers_dir="${1:?}"
	local opt_layer_dir="${layers_dir}/opt"
	local opt_layer_toml="${layers_dir}/opt.toml"

	mkdir -p "${opt_layer_dir}"
	cat >"${opt_layer_toml}" <<-EOF
		[types]
		launch = true
		build = false
		cache = false
	EOF

	cp -a "${CNB_BUILDPACK_DIR}/opt/." "${opt_layer_dir}/"
}

write_launch_toml() {
	local layers_dir="${1:?}"
	local app_dir="${2:?}"

	cat >"${layers_dir}/launch.toml" <<-EOF
		[[processes]]
		type = "web"
		command = "${layers_dir}/opt/run.sh ${app_dir}"
		default = true
	EOF
}
