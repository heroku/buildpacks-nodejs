#!/usr/bin/env bash

fail_on_no_main_file() {
	build_dir=$1
	fn_entry_file=$(json_get_key "$build_dir/package.json" ".main")

	if [[ ! -f $fn_entry_file ]]; then
		error "File at \"main\" in package.json is missing. Check your function and make sure there is a main file."
		exit 1
	fi
}

fail_on_no_main_key() {
	build_dir=$1
	fn_entry_file=$(json_get_key "$build_dir/package.json" ".main")

	if [[ -z $fn_entry_file ]]; then
		error "Key at \"main\" in package.json is missing. Make sure to use \"main\" to specify your root function file."
		exit 1
	fi
}
