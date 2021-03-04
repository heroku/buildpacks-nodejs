#!/usr/bin/env bash

detect_yarn_lock() {
	local build_dir=$1
	[[ -f "$build_dir/yarn.lock" ]]
}

detect_two_lock_files() {
	local build_dir=$1
	[[ -f "$build_dir/package.json" && -f "$build_dir/yarn.lock" ]]
}
