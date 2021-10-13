#!/usr/bin/env bash

set -euo pipefail

app_dir="${1:?}"

exec sf-fx-runtime-nodejs serve "${app_dir}" --host=0.0.0.0 --port="${PORT:-8080}" --debug-port="${DEBUG_PORT:-}"
