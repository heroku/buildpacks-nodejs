#!/usr/bin/env bash
# shellcheck source-path=SCRIPTDIR/..

# Not using -u due to:
# https://github.com/rylnd/shpec/issues/126
set -eo pipefail

bp_dir=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
export CNB_BUILDPACK_DIR="${bp_dir}"

source "${bp_dir}/lib/detect.sh"

create_temp_project_dir() {
	mktemp -dt project_shpec_XXXXX
}

describe "lib/detect.sh"
	describe "detect_function_app"
		it "passes detect when there is a function.toml"
			app_dir="${bp_dir}/fixtures/function-toml"
			detect_function_app "${app_dir}"
			result="${?}"
			assert equal "${result}" 0
		end

		it "passes detect when there is a project.toml"
			app_dir="${bp_dir}/fixtures/project-toml"
			detect_function_app "${app_dir}"
			result="${?}"
			assert equal "${result}" 0
		end

		it "fails detect when there is only a package.json"
			app_dir="${bp_dir}/fixtures/package-json-only"
			set +e
			detect_function_app "${app_dir}"
			result="${?}"
			set -e
			assert equal "${result}" 1
		end
	end

	describe "write_to_build_plan"
		it "writes the build plan to the specified file"
			project_dir=$(create_temp_project_dir)
			buildplan="${project_dir}/buildplan.toml"

			write_to_build_plan "${buildplan}"
			result="${?}"
			assert equal "${result}" 0

			assert file_present "${buildplan}"
			actual=$(cat "${buildplan}")
			expected=$(cat <<-EOF
					[[provides]]
					name = "nodejs-function-runtime"

					[[requires]]
					name = "nodejs-function-runtime"

					[[requires]]
					name = "node"
				EOF
			)
			assert equal "${actual}" "${expected}"
			rm -rf "${project_dir}"
		end
	end
end
