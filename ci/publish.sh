#!/usr/bin/env bash
# SPDX-License-Identifier: Apache-2.0 OR MIT
set -euo pipefail
IFS=$'\n\t'
cd "$(dirname "$0")"/..

# shellcheck disable=SC2154
trap 's=$?; echo >&2 "$0: error on line "${LINENO}": ${BASH_COMMAND}"; exit ${s}' ERR

# A list of paths to the crate to be published.
# It will be published in the order listed.
members=(
    "pin-project-internal"
    "."
)

for i in "${!members[@]}"; do
    (
        set -x
        cd "${members[${i}]}"
        cargo publish
    )
    if [[ $((i + 1)) != "${#members[@]}" ]]; then
        sleep 45
    fi
done
