#!/usr/bin/env bash

download_and_extract_tarball() {
	local -r tarball_url="${1:?}"
	local -r target_directory="${2:?}"
	curl --retry 3 --silent --show-error --fail --max-time 60 --location "${tarball_url}" | tar -xz -C "${target_directory}"
}
