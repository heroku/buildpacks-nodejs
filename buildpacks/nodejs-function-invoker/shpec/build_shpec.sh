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

create_temp_app_dir() {
	mktemp -d -t build_shpec_XXXXX
}

describe "lib/build.sh"
	describe "run_initial_checks"
		it "passes when a main key and file are present"
			project_dir=$(create_temp_app_dir)
			cp -r "${bp_dir}"/fixtures/valid-project/* "$project_dir"

			output=$(run_initial_checks "$project_dir")
			result="${?}"

			assert equal "${result}" 0
		end

		it "fails when no main key present"
			project_dir=$(create_temp_app_dir)
			cp -r "${bp_dir}"/fixtures/package-json-no-main-key/* "$project_dir"

			set +e
			output=$(run_initial_checks "$project_dir")
			result="${?}"
			set -e

			assert equal "${result}" 1
		end

		it "fails when no main file present"
			project_dir=$(create_temp_app_dir)
			cp -r "${bp_dir}"/fixtures/package-json-no-main-file/* "$project_dir"

			set +e
			output=$(run_initial_checks "$project_dir")
			result="${?}"
			set -e

			assert equal "${result}" 1
		end
	end

	describe "install_runtime"
		it "creates a runtime layer containing the runtime"
			stub_command "npm"
			layers_dir=$(create_temp_layers_dir)

			output=$(install_runtime "${layers_dir}")
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
			rm -rf "${layers_dir}"
		end

		it "handles download failures gracefully"
			stub_command "download_file" "return 1"
			layers_dir=$(create_temp_layers_dir)

			set +e
			output=$(install_runtime "${layers_dir}" 2>&1)
			result="${?}"
			set -e

			assert equal "${result}" 1
			assert grep "${output}" "Error: Unable to download the function runtime"
			rm -rf "${layers_dir}"
			unstub_command "download_file"
		end
	end

	describe "write_launch_toml"
		it "configures sf-fx-runtime-nodejs as the web process"
			layers_dir=$(create_temp_layers_dir)
			launch_toml="${layers_dir}/launch.toml"
			app_dir=$(create_temp_app_dir)

			write_launch_toml "${layers_dir}" "${app_dir}"
			result="${?}"

			assert equal "${result}" 0
			assert file_present "${launch_toml}"
			actual=$(cat "${launch_toml}")
			expected=$(cat <<-EOF
					[[processes]]
					type = "web"
					command = "sf-fx-runtime-nodejs serve ${app_dir} -h 0.0.0.0 -p \${PORT:-8080}"
				EOF
			)
			assert equal "${actual}" "${expected}"

			rm -rf "${layers_dir}"
			rm -rf "${app_dir}"
		end
	end
end
