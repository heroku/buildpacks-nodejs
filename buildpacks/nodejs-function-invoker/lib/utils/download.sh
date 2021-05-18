#!/usr/bin/env bash

download_yj() {
	app_dir="$1"

	mkdir -p "${app_dir}/.salesforce_functions/downloads/bin"

	if [[ ! -f "${app_dir}/.salesforce_functions/downloads/bin/yj" ]]; then
		curl -Ls https://github.com/sclevine/yj/releases/download/v2.0/yj-linux >"${app_dir}/.salesforce_functions/downloads/bin/yj" &&
			chmod +x "${app_dir}/.salesforce_functions/downloads/bin/yj"
	fi
}

download_file() {
	local -r url="${1:?}"
	local -r target_path="${2:?}"
	curl --retry 3 --silent --fail --max-time 10 --location "${url}" --output "${target_path}"
}
