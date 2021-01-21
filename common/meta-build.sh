#!/usr/bin/env bash
set -euo pipefail

# Builds a meta buildpack by copying itself to the target directory. Since dependent buildpacks
# might also need to be build before packaging, this script will also look for local buildpack references in
# package.toml, execute their build script if present and modifies the meta-buildpack's package.toml
# (within the target directory) to point to the built version of the of the dependency.

buildpack_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"
target_dir_name="target"
target_dir="${buildpack_dir}/${target_dir_name}"

mkdir "${target_dir}"
rsync -a -L "${buildpack_dir}/" "${target_dir}" --exclude "${target_dir_name}"

if [[ -f "${buildpack_dir}/package.toml" ]]; then
	original_dependency_uris=$(yj -t <"${buildpack_dir}/package.toml" | jq -r .dependencies[].uri)
	for original_dependency_uri in ${original_dependency_uris}; do

		if (cd "${buildpack_dir}" && realpath -q "${original_dependency_uri}"); then
			# Absolute path to the referenced buildpack
			dependency_buildpack_dir="$(cd "${buildpack_dir}" && realpath "${original_dependency_uri}")"

			if [[ -d "${dependency_buildpack_dir}" && -f "${dependency_buildpack_dir}/build.sh" ]]; then
				echo "Building buildpack at ${dependency_buildpack_dir}..."
				"${dependency_buildpack_dir}/build.sh"
				echo "Build complete!"

				updated_dependency_uri="$(
					realpath --relative-to "${target_dir}" "${dependency_buildpack_dir}/target"
				)"

				jq_filter=".dependencies |= map(select(.uri == \"${original_dependency_uri}\").uri |= \"${updated_dependency_uri}\")"

				updated_package_toml=$(yj -t <"${target_dir}/package.toml" | jq "${jq_filter}" | yj -jt)
				echo "${updated_package_toml}" >"${target_dir}/package.toml"
			fi
		fi
	done
fi
