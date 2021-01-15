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

describe "lib/build.sh"
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
