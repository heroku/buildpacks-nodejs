#!/usr/bin/env bash

set -e
set -o pipefail

shpec_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"

# shellcheck source=SCRIPTDIR/../lib/utils/json.sh
source "${shpec_dir}/../lib/utils/json.sh"
# shellcheck source=SCRIPTDIR/../lib/utils/log.sh
source "${shpec_dir}/../lib/utils/log.sh"
# shellcheck source=SCRIPTDIR/../lib/utils/toml.sh
source "${shpec_dir}/../lib/utils/toml.sh"
# shellcheck source=SCRIPTDIR/../lib/bootstrap.sh
source "${shpec_dir}/../lib/bootstrap.sh"
# shellcheck source=SCRIPTDIR/../lib/build.sh
source "${shpec_dir}/../lib/build.sh"

create_temp_layer_dir() {
	mktemp -d -t build_shpec_XXXXX
}

create_temp_project_dir() {
	mktemp -u -t project_shpec_XXXXX
}

create_temp_package_json() {
	mkdir -p "tmp"
	cp "./fixtures/package-patch-versions.json" "tmp/package.json"
}

rm_temp_dirs() {
	rm -rf "$1"
	rm -rf "tmp"
}

create_binaries() {
	stub_command "echo"
	bootstrap_buildpack "$1"
	unstub_command "echo"
}

rm_binaries() {
	rm -f "$bp_dir/bin/resolve-version"
}

describe "lib/build.sh"
	stub_command "info"
	stub_command "node"
	rm_binaries

	layers_dir=$(create_temp_layer_dir)

	describe "clear_cache_on_stack_change"
		touch "$layers_dir/my_layer.toml"

		export CNB_STACK_ID="heroku-20"

		it "does not delete layers with same stack"
			assert file_present "$layers_dir/my_layer.toml"

			clear_cache_on_stack_change "$layers_dir"

			assert file_present "$layers_dir/my_layer.toml"
		end

		write_to_store_toml "$layers_dir"

		it "deletes layers when stack changes"
			CNB_STACK_ID="heroku-22"

			assert file_present "$layers_dir/my_layer.toml"

			clear_cache_on_stack_change "$layers_dir"

			assert file_absent "$layers_dir/my_layer.toml"
		end

		unset CNB_STACK_ID
	end

	describe "clear_cache_on_node_version_change"

		touch "$layers_dir/yarn"

		it "does not delete layers with same node version"
			mkdir "${layers_dir}/nodejs"
			mkdir "${layers_dir}/nodejs/env.build"
			
			echo -e "$(node -v)\c" >>"${layers_dir}/nodejs/env.build/PREV_NODE_VERSION"

			assert file_present "$layers_dir/yarn"

			clear_cache_on_node_version_change "$layers_dir" "$layers_dir/nodejs"

			assert file_present "$layers_dir/yarn"
		end

		it "deletes layers when node version changes"
			rm -rf "${layers_dir}/nodejs/env.build/PREV_NODE_VERSION"
			echo -e "different_version" >>"${layers_dir}/nodejs/env.build/PREV_NODE_VERSION"

			assert file_present "$layers_dir/yarn"

			clear_cache_on_node_version_change "$layers_dir" "$layers_dir/nodejs"

			assert file_absent "$layers_dir/yarn"
		end

	end

	describe "write_to_store_toml"

		if [[ -s "$layers_dir/store.toml" ]]; then
			rm -rf "$layers_dir/store.toml"
		fi

		it "creates store.toml when not present"
			assert file_absent "$layers_dir/store.toml"

			write_to_store_toml "$layers_dir"

			assert file_present "$layers_dir/store.toml"
		end
	end

	describe "boostrap_buildpack"
		create_binaries "$layers_dir/bootstrap"

		it "does not write to bin"
			assert file_absent "bin/resolve-version"
		end

		it "creates layered bootstrap binaries"
			assert file_present "$layers_dir/bootstrap/bin/resolve-version"
		end
	end

	describe "set_node_modules_path"
		layers_dir=$(create_temp_layer_dir)
		it "sets NODE_MODULES_PATH to node modules directory path"
			assert file_absent "$layers_dir/nodejs/env/NODE_MODULES_PATH"

			set_node_modules_path "$layers_dir/nodejs"

			assert file_present "$layers_dir/nodejs/env/NODE_MODULES_PATH"
		end
	end

	describe "install_or_reuse_toolbox"
		export PATH=$layers_dir/toolbox/bin:$PATH

		it "creates a toolbox layer"
			install_or_reuse_toolbox "$layers_dir/toolbox"

			assert file_present "$layers_dir/toolbox/bin/yj"
		end

		it "creates a toolbox.toml"
			install_or_reuse_toolbox "$layers_dir/toolbox"

			assert file_present "$layers_dir/toolbox.toml"
		end
	end

	describe "store_node_version"
		layers_dir=$(create_temp_layer_dir)

		touch "${layers_dir}/nodejs.toml"
		echo -e "[metadata]\nversion = \"test_version\"" > "${layers_dir}/nodejs.toml"

		it "stores node version in PREV_NODE_VERSION env"
			assert file_absent "$layers_dir/nodejs/env.build/PREV_NODE_VERSION.override"
			store_node_version "$layers_dir/nodejs"
			assert equal "$(cat "$layers_dir/nodejs/env.build/PREV_NODE_VERSION.override")" test_version
		end
	end

	describe "install_or_reuse_node"
		layers_dir=$(create_temp_layer_dir)
		project_dir=$(create_temp_project_dir)
		create_binaries "$layers_dir/bootstrap"

		it "creates a node layer when it does not exist"
			assert file_absent "$layers_dir/nodejs/bin/node"
			assert file_absent "$layers_dir/nodejs/bin/npm"

			install_or_reuse_node "$layers_dir/nodejs" "$project_dir"

			assert file_present "$layers_dir/nodejs/bin/node"
			assert file_present "$layers_dir/nodejs/bin/npm"
		end

		it "reuses node layer when versions match"
			# TODO: set up fixtures for version matching
		end
	end

	describe "parse_package_json_engines"
		layers_dir=$(create_temp_layer_dir)

		create_binaries "$layers_dir/bootstrap"

		echo -e "[metadata]\n" > "${layers_dir}/package_manager_metadata.toml"
		create_temp_package_json

		parse_package_json_engines "$layers_dir/package_manager_metadata" "tmp"

		it "writes npm version to layers/node.toml"
			npm_version=$(toml_get_key_from_metadata "$layers_dir/package_manager_metadata.toml" "npm_version")

			assert equal "6.9.1" "$npm_version"
		end

		it "writes yarn_version to layers/node.toml"
			stub_command "echo"
			yarn_version=$(toml_get_key_from_metadata "$layers_dir/package_manager_metadata.toml" "yarn_version")

			assert equal "1.19.1" "$yarn_version"
		end

		rm_temp_dirs "$layers_dir"
	end

	describe "install_or_reuse_yarn"
		layers_dir=$(create_temp_layer_dir)
		project_dir=$(create_temp_project_dir)

		create_binaries "$layers_dir/bootstrap"

		it "creates a yarn layer when it does not exist"
			assert file_absent "$layers_dir/yarn/bin/yarn"

			install_or_reuse_yarn "$layers_dir/yarn" "$project_dir"

			assert file_present "$layers_dir/yarn/bin/yarn"
		end

		it "reuses yarn layer when versions match"
			# TODO: set up fixtures for version matching
		end

		rm_temp_dirs "$layers_dir"
	end

	describe "set_node_env"
		layers_dir=$(create_temp_layer_dir)
		it "sets env/NODE_ENV.override to production when NODE_ENV is blank"
			assert file_absent "$layers_dir/nodejs/env/NODE_ENV.override"

			set_node_env "$layers_dir/nodejs"

			assert file_present "$layers_dir/nodejs/env/NODE_ENV.override"
			assert equal "$(cat "$layers_dir/nodejs/env/NODE_ENV.override")" production

			rm "$layers_dir/nodejs/env/NODE_ENV.override"
		end

		it "sets env/NODE_ENV.override to NODE_ENV"
			export NODE_ENV="test"

			set_node_env "$layers_dir/nodejs"

			assert file_present "$layers_dir/nodejs/env/NODE_ENV.override"
			assert equal "$(cat "$layers_dir/nodejs/env/NODE_ENV.override")" test

			unset NODE_ENV
		end

		rm_temp_dirs "$layers_dir"
	end

	describe "copy_profile"
		layers_dir=$(create_temp_layer_dir)
		it "copies WEB_CONCURRENCY.sh script"
			assert file_absent "$layers_dir/nodejs/profile.d/WEB_CONCURRENCY.sh"

			copy_profile "$layers_dir/nodejs" "$bp_dir"

			assert file_present "$layers_dir/nodejs/profile.d/WEB_CONCURRENCY.sh"
		end

		rm_temp_dirs "$layers_dir"
	end

	describe "write_launch_toml"
		layers_dir=$(create_temp_layer_dir)

		create_binaries "$layers_dir/bootstrap"

		mkdir -p "tmp"
		touch "tmp/server.js" "tmp/index.js"

		it "creates a launch.toml file when there is index.js and server.js"
			assert file_absent "$layers_dir/launch.toml"

			write_launch_toml "tmp" "$layers_dir/launch.toml"

			assert file_present "$layers_dir/launch.toml"

			rm "$layers_dir/launch.toml"
		end

		it "creates a launch.toml file when there is server.js and no index.js"
			rm "tmp/index.js"

			assert file_absent "$layers_dir/launch.toml"

			write_launch_toml "tmp" "$layers_dir/launch.toml"

			assert file_present "$layers_dir/launch.toml"

			rm "$layers_dir/launch.toml"
		end

		it "does not create launch.toml when no js initialize files"
			rm "tmp/server.js"

			assert file_absent "$layers_dir/launch.toml"

			write_launch_toml "tmp" "$layers_dir/launch.toml"

			assert file_absent "$layers_dir/launch.toml"
		end

		rm_temp_dirs "$layers_dir"
	end

	unstub_command "info"
	unstub_command "node"
	rm_binaries
end

cd ../..
