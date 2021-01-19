#!/usr/bin/env bash

set -e
set -o pipefail

shpec_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"

# shellcheck source=SCRIPTDIR/../lib/detect.sh
source "${shpec_dir}/../lib/detect.sh"

create_temp_project_dir() {
	mktemp -dt project_shpec_XXXXX
}

describe "lib/detect.sh"
	describe "detect_package_json"
		project_dir=$(create_temp_project_dir)

		it "detects if there is no package.json"
			set +e
			detect_package_json "$project_dir"
			loc_var=$?
			set -e

			assert equal $loc_var 1
		end

		it "detects if there is package.json"
			touch "$project_dir/package.json"

			detect_package_json "$project_dir"

			assert equal "$?" 0
		end

		rm -rf "$project_dir"
	end

	describe "write_to_build_plan"
		it "writes node and node_modules as expected in build plan"
			project_dir=$(create_temp_project_dir)
			touch "$project_dir/buildplan.toml"
			write_to_build_plan "$project_dir/buildplan.toml"
			actual=$(cat "$project_dir/buildplan.toml")
			echo "$actual"
			expected=$(cat <<EOF
	[[provides]]
	name = "node_modules"

	[[requires]]
	name = "node_modules"

	[[requires]]
	name = "node"
EOF
)
			assert equal "$actual" "$expected"
		end
	end
end
