#!/usr/bin/env bash
set -euo pipefail

# This script is an integral part of the release workflow: .github/workflows/release.yml
# It requires the following environment variables to function correctly:
#
# REQUESTED_BUILDPACK_ID - The ID of the buildpack to package and push to the container registry.

while IFS="" read -r -d "" buildpack_toml_path; do
	buildpack_id="$(yj -t <"${buildpack_toml_path}" | jq -r .buildpack.id)"
	buildpack_version="$(yj -t <"${buildpack_toml_path}" | jq -r .buildpack.version)"
	buildpack_docker_repository="$(yj -t <"${buildpack_toml_path}" | jq -r .metadata.release.docker.repository)"
	buildpack_path=$(dirname "${buildpack_toml_path}")

	if [[ $buildpack_id == "${REQUESTED_BUILDPACK_ID}" ]]; then
		cnb_shim_tarball_url="https://github.com/heroku/cnb-shim/releases/download/v0.3/cnb-shim-v0.3.tgz"
		cnb_shim_tarball_sha256="109cfc01953cb04e69c82eec1c45c7c800bd57d2fd0eef030c37d8fc37a1cb4d"
		local_cnb_shim_tarball=$(mktemp)

		v2_buildpack_tarball_url="$(yj -t <"${buildpack_toml_path}" | jq -r ".metadata.shim.tarball // empty")"
		v2_buildpack_tarball_sha256="$(yj -t <"${buildpack_toml_path}" | jq -r ".metadata.shim.sha256 // empty")"
		local_v2_buildpack_tarball=$(mktemp)

		# If the buildpack has a V2 buildpack tarball in its metadata it's supposed to be a shimmed buildpack.
		# We download the shim and the V2 buildpack to the buildpack directory, turning it into a CNB. This
		# transformation is transparent for the code that follows after it.
		if [[ -n "${v2_buildpack_tarball_url:-}" ]]; then
			curl --retry 3 --location "${cnb_shim_tarball_url}" --output "${local_cnb_shim_tarball}"
			curl --retry 3 --location "${v2_buildpack_tarball_url}" --output "${local_v2_buildpack_tarball}"

			if ! echo "${cnb_shim_tarball_sha256} ${local_cnb_shim_tarball}" | sha256sum --check --status; then
				echo "Checksum verification of cnb_shim failed!"
				exit 1
			fi

			if ! echo "${v2_buildpack_tarball_sha256} ${local_v2_buildpack_tarball}" | sha256sum --check --status; then
				echo "Checksum verification of V2 buildpack tarball failed!"
				exit 1
			fi

			mkdir -p "${buildpack_path}/target"
			tar -xzmf "${local_cnb_shim_tarball}" -C "${buildpack_path}"
			tar -xzmf "${local_v2_buildpack_tarball}" -C "${buildpack_path}/target"
		fi

		image_name="${buildpack_docker_repository}:${buildpack_version}"
		pack package-buildpack --config "${buildpack_path}/package.toml" --publish "${image_name}"

		# We might have local changes after shimming the buildpack. To ensure scripts down the pipeline work with
		# a clean state, we reset all local changes here.
		git reset --hard
		git clean -fdx

		echo "::set-output name=id::${buildpack_id}"
		echo "::set-output name=version::${buildpack_version}"
		echo "::set-output name=path::${buildpack_path}"
		echo "::set-output name=address::${buildpack_docker_repository}@$(crane digest "${image_name}")"
		exit 0
	fi
done < <(find . -name buildpack.toml -print0)

echo "Could not find requested buildpack with id ${REQUESTED_BUILDPACK_ID}!"
exit 1
