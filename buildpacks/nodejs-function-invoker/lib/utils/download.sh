#!/usr/bin/env bash

install_yj() {
	build_dir=$1

	mkdir -p "${build_dir}/bin"

	if [[ ! -f "${build_dir}/bin/yj" ]]; then
		curl -Ls https://github.com/sclevine/yj/releases/download/v2.0/yj-linux >"${build_dir}/bin/yj" &&
			chmod +x "${build_dir}/bin/yj"
	fi
}

download_file() {
	local -r url="${1:?}"
	local -r target_path="${2:?}"
	curl --retry 3 --silent --fail --max-time 10 --location "${url}" --output "${target_path}"
}
