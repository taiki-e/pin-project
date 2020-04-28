#!/bin/bash

# A script to run a simplified version of the checks done by CI.
#
# Usage
#
# ```sh
# . ./ci.sh
# ```

echo "Running 'cargo fmt -- --check'"
cargo +nightly fmt --all -- --check

echo "Running 'cargo clippy'"
cargo +nightly clippy --all --all-features

echo "Running 'cargo test'"
cargo +nightly test --all --all-features

echo "Running 'cargo doc'"
cargo +nightly doc --no-deps --all --all-features

echo "Running 'compiletest'"
. ./compiletest.sh

# See also https://docs.rs/macrotest/1/macrotest/#updating-expandedrs
echo "Running 'expandtest'"
cargo +nightly test --manifest-path tests/expand/Cargo.toml
