#!/usr/bin/env bash
# shellcheck source-path=SCRIPTDIR/..

# Not using -u due to:
# https://github.com/rylnd/shpec/issues/126
set -eo pipefail

bp_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
export CNB_BUILDPACK_DIR="${bp_dir}"

source "${bp_dir}/lib/build.sh"

create_temp_layers_dir() {
	mktemp -d -t layers_shpec_XXXXX
}

describe "lib/build.sh"
	describe "install_runtime"
		it "creates a runtime layer containing the runtime"
			layers_dir=$(create_temp_layers_dir)

			install_runtime "${layers_dir}" > /dev/null
			result="${?}"

			assert equal "${result}" 0

			assert file_present "${layers_dir}/sf-fx-runtime-nodejs.toml"
			actual_layer_manifest=$(cat "${layers_dir}/sf-fx-runtime-nodejs.toml")
			expected_layer_manifest=$(cat <<-EOF
					[types]
					launch = true
					build = false
					cache = false
				EOF
			)
			assert equal "${actual_layer_manifest}" "${expected_layer_manifest}"

			# TODO: Switch to `bin/sf-fx-runtime-nodejs` when buildpack.toml contains the real archive URL.
			assert file_present "${layers_dir}/sf-fx-runtime-nodejs/node-v14.16.1-linux-x64/bin/node"

			rm -rf "${layers_dir}"
		end

		it "handles download failures gracefully"
			stub_command "download_and_extract_tarball" "return 1"
			layers_dir=$(create_temp_layers_dir)

			set +e
			output=$(install_runtime "${layers_dir}" 2>&1)
			result="${?}"
			set -e

			assert equal "${result}" 1
			assert grep "${output}" "Error: Unable to download the function runtime"
			rm -rf "${layers_dir}"
			unstub_command "download_and_extract_tarball"
		end
	end

	describe "write_launch_toml"
		it "configures sf-fx-runtime-nodejs as the web process"
			layers_dir=$(create_temp_layers_dir)
			launch_toml="${layers_dir}/launch.toml"

			write_launch_toml "${layers_dir}"
			result="${?}"

			assert equal "${result}" 0
			assert file_present "${launch_toml}"
			actual=$(cat "${launch_toml}")
			expected=$(cat <<-EOF
					[[processes]]
					type = "web"
					command = "sf-fx-runtime-nodejs ."
				EOF
			)
			assert equal "${actual}" "${expected}"

			rm -rf "${layers_dir}"
		end
	end
end
