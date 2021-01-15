#!/usr/bin/env bash

set -e
set -o pipefail

source "./lib/detect.sh"

create_temp_project_dir() {
  mktemp -dt project_shpec_XXXXX
}

describe "lib/detect.sh"
  describe "detect_tsconfig_json"
    it "exits with 1 if there is no tsconfig.json"
      project_dir=$(create_temp_project_dir)

      set +e
      detect_tsconfig_json "$project_dir"
      loc_var=$?
      set -e

      assert equal "$loc_var" 1
    end

    it "exits with 0 if there is tsconfig.json"
      project_dir=$(create_temp_project_dir)
      touch "$project_dir/tsconfig.json"

      detect_tsconfig_json "$project_dir"

      assert equal "$?" 0
    end
  end
