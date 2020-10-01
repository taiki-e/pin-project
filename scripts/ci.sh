#!/bin/bash

# A script to run a simplified version of the checks done by CI.
#
# Usage:
#     bash scripts/ci.sh
#

set -euo pipefail

echo "Running 'cargo fmt'"
cargo +nightly fmt --all

echo "Running 'cargo clippy'"
cargo +nightly clippy --all --all-features --all-targets

echo "Running 'cargo test'"
TRYBUILD=overwrite cargo +nightly test --all --all-features --exclude expandtest

echo "Running 'cargo doc'"
cargo +nightly doc --no-deps --all --all-features

echo "Running 'expandtest'"
"$(cd "$(dirname "${0}")" && pwd)"/expandtest.sh
