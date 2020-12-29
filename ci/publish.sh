#!/bin/bash

set -euo pipefail
IFS=$'\n\t'

# A list of paths to the crate to be published.
# It will be published in the order listed.
MEMBERS=(
  "pin-project-internal"
  "."
)

function error {
  echo "error: $*" >&2
}

function retry() {
  local -i max_retry=${1}
  local -i count=0
  while ! eval "${2}"; do
    ((count++))
    if ((count > max_retry)); then
      error "${3}"
      exit 1
    fi
    echo "info: retry after $((10 * count)) seconds"
    sleep $((10 * count))
  done
}

cd "$(cd "$(dirname "${0}")" && pwd)"/..

# publishing
for member in "${MEMBERS[@]}"; do
  (
    cd "${member}"
    pwd
    echo "info: running 'cargo publish ${dryrun:-}'"
    retry 2 "cargo publish ${dryrun:-}" "unable to publish ${member}"
  )
done
