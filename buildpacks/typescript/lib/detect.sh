#!/usr/bin/env bash

detect_tsconfig_json() {
  local build_dir=$1
  [[ -f "$build_dir/tsconfig.json" ]]
}
