#!/usr/bin/env bash

set -e
set -o pipefail

shpec_dir="$(cd "$(dirname "${BASH_SOURCE[0]}")" >/dev/null 2>&1 && pwd)"

# shellcheck source=SCRIPTDIR/../lib/utils/log.sh
source "${shpec_dir}/../lib/utils/log.sh"
# shellcheck source=SCRIPTDIR/../lib/build.sh
source "${shpec_dir}/../lib/build.sh"

create_temp_project_dir() {
  mktemp -dt project_shpec_XXXXX
}

create_temp_layer_dir() {
	mktemp -d -t layer_shpec_XXXXX
}

describe "lib/build.sh"

  layers_dir=$(create_temp_layer_dir)

  describe "clear_cache_on_stack_change"
    touch "$layers_dir/my_layer.toml"

    export CNB_STACK_ID="heroku-20"

    it "creates store.toml when not present"
      assert file_absent "$layers_dir/store.toml"

      clear_cache_on_stack_change "$layers_dir"

      assert file_present "$layers_dir/store.toml"
    end

    it "does not delete layers with same stack"
      assert file_present "$layers_dir/my_layer.toml"

      clear_cache_on_stack_change "$layers_dir"

      assert file_present "$layers_dir/my_layer.toml"
    end

    it "deletes layers when stack changes"
      CNB_STACK_ID="heroku-22"

      assert file_present "$layers_dir/my_layer.toml"

      clear_cache_on_stack_change "$layers_dir"

      assert file_absent "$layers_dir/my_layer.toml"
    end

    unset CNB_STACK_ID
  end

  describe "store_node_version"
    layers_dir=$(create_temp_layer_dir)

    touch "${layers_dir}/store.toml"
    echo -e "[metadata]\nnode_version = \"test_version\"" > "${layers_dir}/store.toml"

    it "stores node version in PREV_NODE_VERSION env"
      assert file_absent "$layers_dir/env/PREV_NODE_VERSION"
      store_node_version "$layers_dir"
      assert equal "$(cat "$layers_dir/env/PREV_NODE_VERSION")" test_version
    end
  end

  describe "clear_cache_on_node_version_change"

		touch "$layers_dir/node_modules"

		it "does not delete layers with same node version"
			echo -e "$(echo $(node -v))" >>"${layers_dir}/env/PREV_NODE_VERSION"

			assert file_present "$layers_dir/node_modules"

			clear_cache_on_stack_change "$layers_dir"

			assert file_present "$layers_dir/node_modules"
		end

		it "deletes layers when stack changes"
			rm -rf "${layers_dir}/env/PREV_NODE_VERSION"
			echo -e "$(echo $(node -v))" >>"${layers_dir}/env/PREV_NODE_VERSION"

			assert file_present "$layers_dir/node_modules"

			clear_cache_on_stack_change "$layers_dir"

			assert file_absent "$layers_dir/node_modules"
		end

	end

  describe "detect_out_dir"
    it "exits with 1 if there is no outDir directory"
      project_dir=$(create_temp_project_dir)

      set +e
      detect_out_dir "$project_dir"
      loc_var=$?
      set -e

      assert equal "$loc_var" 1
    end

    it "exits with 0 if there is outDir directory"
      # TODO: fix when we have a better plan for cross-buildpack dependencies

      # project_dir=$(create_temp_project_dir)

      # touch "$project_dir/dist"
      # cp "./fixtures/tsconfig.json" "$project_dir"

      # detect_out_dir "$project_dir"

      # assert equal "$?" 0
    end
  end
end
