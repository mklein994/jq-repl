#!/bin/bash

SCRIPT_DIR="${ dirname "${ readlink -f "$0";}";}"

gojq -nrf /dev/stdin <<'JQ' >"${SCRIPT_DIR}/commands.txt"
  [
    ("prelude"
    | modulemeta.deps[]
    | select(.is_data or has("as") | not).relpath
    | modulemeta.defs),

    builtins
  ]
  | add
  | map(scan("^[^/]+"))
  | unique[]
JQ
