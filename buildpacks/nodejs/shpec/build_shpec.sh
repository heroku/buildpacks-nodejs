#!/usr/bin/env bash

set -e
set -o pipefail

source "./lib/utils/json.sh"
source "./lib/utils/log.sh"
source "./lib/utils/toml.sh"
source "./lib/bootstrap.sh"
source "./lib/build.sh"

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
  rm_binaries

  layers_dir=$(create_temp_layer_dir)

  describe "boostrap_buildpack"
    create_binaries "$layers_dir/bootstrap"

    it "does not write to bin"
      assert file_absent "bin/resolve-version"
    end

    it "creates layered bootstrap binaries"
      assert file_present "$layers_dir/bootstrap/bin/resolve-version"
    end
  end

  describe "set_up_environment"
    layers_dir=$(create_temp_layer_dir)
    it "sets env.build/NODE_ENV.override to production when NODE_ENV is blank"
      assert file_absent "$layers_dir/nodejs/env.build/NODE_ENV.override"

      set_up_environment "$layers_dir/nodejs"

      assert file_present "$layers_dir/nodejs/env.build/NODE_ENV.override"
      assert equal "$(cat "$layers_dir/nodejs/env.build/NODE_ENV.override")" production

      rm "$layers_dir/nodejs/env.build/NODE_ENV.override"
    end

    it "sets env.build/NODE_ENV.override to NODE_ENV"
      export NODE_ENV="test"

      set_up_environment "$layers_dir/nodejs"

      assert file_present "$layers_dir/nodejs/env.build/NODE_ENV.override"
      assert equal "$(cat "$layers_dir/nodejs/env.build/NODE_ENV.override")" test

      unset NODE_ENV
    end

    rm_temp_dirs "$layers_dir"
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
    it "sets env.launch/NODE_ENV.override to production when NODE_ENV is blank"
      assert file_absent "$layers_dir/nodejs/env.launch/NODE_ENV.override"

      set_node_env "$layers_dir/nodejs"

      assert file_present "$layers_dir/nodejs/env.launch/NODE_ENV.override"
      assert equal "$(cat "$layers_dir/nodejs/env.launch/NODE_ENV.override")" production

      rm "$layers_dir/nodejs/env.launch/NODE_ENV.override"
    end

    it "sets env.launch/NODE_ENV.override to NODE_ENV"
      export NODE_ENV="test"

      set_node_env "$layers_dir/nodejs"

      assert file_present "$layers_dir/nodejs/env.launch/NODE_ENV.override"
      assert equal "$(cat "$layers_dir/nodejs/env.launch/NODE_ENV.override")" test

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
  rm_binaries
end
