#!/usr/bin/env bash

set -euo pipefail

runtime_bin=$1
function_dir=$2

exec $runtime_bin serve "$function_dir" --workers 2 --host "::" --port "${PORT:-8080}" --debug-port "${DEBUG_PORT:-}"
