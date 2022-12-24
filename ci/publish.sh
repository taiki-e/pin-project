#!/bin/bash
set -euo pipefail
IFS=$'\n\t'
cd "$(dirname "$0")"/..

# shellcheck disable=SC2154
trap 's=$?; echo >&2 "$0: Error on line "${LINENO}": ${BASH_COMMAND}"; exit ${s}' ERR

# A list of paths to the crate to be published.
# It will be published in the order listed.
MEMBERS=(
    "pin-project-internal"
    "."
)

for i in "${!MEMBERS[@]}"; do
    (
        set -x
        cd "${MEMBERS[${i}]}"
        cargo publish
    )
    if [[ $((i + 1)) != "${#MEMBERS[@]}" ]]; then
        sleep 45
    fi
done
