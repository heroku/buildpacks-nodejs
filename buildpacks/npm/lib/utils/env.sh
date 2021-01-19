#!/usr/bin/env bash

export_env() {
	local env_dir=${1:-$ENV_DIR}
	local whitelist=${2:-''}
	local blacklist
	blacklist="$(_env_blacklist "$3")"
	if [ -d "$env_dir" ]; then
		# Environment variable names won't contain characters affected by:
		# shellcheck disable=SC2045
		for e in $(ls "$env_dir"); do
			echo "$e" | grep -E "$whitelist" | grep -qvE "$blacklist" &&
				export "$e=$(cat "$env_dir/$e")"
			:
		done
	fi
}

# Usage: $ _env-blacklist pattern
# Outputs a regex of default blacklist env vars.
_env_blacklist() {
	local regex=${1:-''}
	if [ -n "$regex" ]; then
		regex="|$regex"
	fi
	echo "^(PATH|GIT_DIR|CPATH|CPPATH|LD_PRELOAD|LIBRARY_PATH$regex)$"
}
