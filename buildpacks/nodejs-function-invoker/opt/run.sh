#!/usr/bin/env bash

set -euo pipefail

app_dir="${1:?}"

if [[ -n "${DEBUG_PORT:-}" ]]; then
	# Adds a space separator only if NODE_OPTIONS was already set.
	export NODE_OPTIONS="${NODE_OPTIONS}${NODE_OPTIONS:+ }--inspect=0.0.0.0:${DEBUG_PORT}"
fi

exec sf-fx-runtime-nodejs serve "${app_dir}" --host 0.0.0.0 --port "${PORT:-8080}"
