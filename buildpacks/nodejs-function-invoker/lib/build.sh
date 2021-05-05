#!/usr/bin/env bash
# shellcheck source-path=SCRIPTDIR/..

source "${CNB_BUILDPACK_DIR}/lib/utils/download.sh"
source "${CNB_BUILDPACK_DIR}/lib/utils/log.sh"
source "${CNB_BUILDPACK_DIR}/lib/utils/toml.sh"

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

	runtime_tarball_url=$(toml_get_key_from_metadata "${CNB_BUILDPACK_DIR}/buildpack.toml" "runtime.url")

	info "Starting download of function runtime"

	if ! download_and_extract_tarball "${runtime_tarball_url}" "${runtime_layer_dir}"; then
		# Ideally this would be a multi-line error message explaining that the cause is
		# likely network issues and to retry, but error() doesn't support multi-line.
		error "Unable to download the function runtime from ${runtime_tarball_url}"
		exit 1
	fi

	info "Function runtime installation successful"
}

write_launch_toml() {
	local layers_dir="${1:?}"

	cat >"${layers_dir}/launch.toml" <<-EOF
		[[processes]]
		type = "web"
		command = "sf-fx-runtime-nodejs ."
	EOF
}
