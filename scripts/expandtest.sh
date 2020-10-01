#!/bin/bash

# A script to run expandtest.
#
# Usage:
#     bash scripts/expandtest.sh
#

set -euo pipefail

script_dir="$(cd "$(dirname "${0}")" && pwd)"

if [[ "${CI:-false}" != "true" ]]; then
    # See also https://docs.rs/macrotest/1/macrotest/#updating-expandedrs
    rm -rf "${script_dir}"/../tests/expand/tests/expand/*.expanded.rs
fi

cargo +nightly test --manifest-path "${script_dir}"/../tests/expand/Cargo.toml
