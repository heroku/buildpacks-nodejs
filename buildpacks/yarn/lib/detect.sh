#!/usr/bin/env bash

detect_yarn_lock() {
	local build_dir=$1
	[[ -f "$build_dir/yarn.lock" ]]
}
