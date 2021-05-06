#!/usr/bin/env bash

download_file() {
	local -r url="${1:?}"
	local -r target_path="${2:?}"
	curl --retry 3 --silent --fail --max-time 10 --location "${url}" --output "${target_path}"
}
