#!/bin/bash

# A script to run expandtest.
#
# Usage:
#     bash scripts/expandtest.sh
#

set -euo pipefail

script_dir="$(cd "$(dirname "${0}")" && pwd)"

if [[ "${CI:-false}" != "true" ]]; then
    toolchain="+nightly"

    # First, check if the compile fails for another reason.
    cargo ${toolchain} check --tests --manifest-path "${script_dir}"/../tests/expand/Cargo.toml

    # Next, remove the `*.expanded.rs` files to allow updating those files.
    # Refs: https://docs.rs/macrotest/1/macrotest/#updating-expandedrs
    rm -rf "${script_dir}"/../tests/expand/tests/expand/*.expanded.rs
fi

cargo ${toolchain:-} test --manifest-path "${script_dir}"/../tests/expand/Cargo.toml
