#!/usr/bin/env bash

toml_get_key_from_metadata() {
  local file="$1"
  local key="$2"

  if test -f "$file"; then
    yj -t < "${file}" | jq -r ".metadata.${key}"
  else
    echo ""
  fi
}
