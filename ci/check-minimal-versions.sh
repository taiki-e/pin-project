#!/bin/bash
# Check all public crates with minimal version dependencies.
#
# Note that this script modifies Cargo.toml and Cargo.lock and does not restore
# them after a run.
#
# Refs:
# * minimal versions: https://github.com/rust-lang/cargo/issues/5657
# * features 2.0: https://github.com/rust-lang/cargo/issues/8088

set -euo pipefail

# Remove dev-dependencies from Cargo.toml to prevent the next `cargo update`
# from determining minimal versions based on dev-dependencies.
cargo hack --remove-dev-deps --workspace
# Update Cargo.lock to minimal version dependencies.
cargo update -Zminimal-versions
# Run check for all public members of the workspace.
cargo hack check --workspace --all-features --ignore-private -Zfeatures=all
