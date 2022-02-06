#!/bin/bash
set -euxo pipefail
IFS=$'\n\t'
cd "$(dirname "$0")"/..

# A list of paths to the crate to be published.
# It will be published in the order listed.
MEMBERS=(
    "pin-project-internal"
    "."
)

for i in "${!MEMBERS[@]}"; do
    (
        cd "${MEMBERS[${i}]}"
        cargo publish
    )
    if [[ $((i + 1)) != "${#MEMBERS[@]}" ]]; then
        sleep 45
    fi
done
