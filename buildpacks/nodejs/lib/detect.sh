#!/usr/bin/env bash

detect_package_json() {
  local build_dir=$1
  [[ -f "$build_dir/package.json" ]]
}

write_to_build_plan() {
  local build_plan=$1
  cat << EOF > "$build_plan"
  [[provides]]
  name = "node"

  [[requires]]
  name = "node"
EOF
}
