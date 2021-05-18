#!/usr/bin/env bash

download_yj() {
	bp_dir=$1

	mkdir -p "${bp_dir}/downloads/bin"

	if [[ ! -f "${bp_dir}/downloads/bin/yj" ]]; then
		curl -Ls https://github.com/sclevine/yj/releases/download/v2.0/yj-linux >"${bp_dir}/downloads/bin/yj" &&
			chmod +x "${bp_dir}/downloads/bin/yj"
	fi
}

download_file() {
	local -r url="${1:?}"
	local -r target_path="${2:?}"
	curl --retry 3 --silent --fail --max-time 10 --location "${url}" --output "${target_path}"
}
