#!/bin/bash

# Run a simplified version of the checks done by CI.
#
# Usage:
#     ./scripts/ci.sh [+toolchain]
#
# Note: This script requires nightly Rust, rustfmt, clippy, and cargo-expand

set -euo pipefail
IFS=$'\n\t'

error() {
    echo "error: $*" >&2
}
warn() {
    echo "warning: $*" >&2
}

# Decide Rust toolchain. Nightly is used by default.
toolchain="+nightly"
if [[ "${1:-}" == "+"* ]]; then
    toolchain="${1}"
    shift
fi
# Make sure toolchain is installed.
if ! cargo "${toolchain}" -V &>/dev/null; then
    rustup toolchain install "${toolchain#+}" --no-self-update --profile minimal
fi

if [[ "${toolchain:-+nightly}" != "+nightly"* ]]; then
    error "ci.sh requires nightly Rust"
    exit 1
fi
if ! rustup "${toolchain}" component add rustfmt &>/dev/null \
    || ! cargo expand -V &>/dev/null; then
    warn "ci.sh requires rustfmt and cargo-expand to run all tests"
fi

# Run rustfmt.
if ! rustup "${toolchain}" component add rustfmt &>/dev/null; then
    warn "component 'rustfmt' is unavailable for toolchain '${toolchain#+}'"
else
    (
        set -x
        cargo "${toolchain}" fmt --all
    )
fi

# Run clippy.
if ! rustup "${toolchain}" component add clippy &>/dev/null; then
    warn "component 'clippy' is unavailable for toolchain '${toolchain#+}'"
else
    (
        set -x
        cargo "${toolchain}" clippy --all --all-features --all-targets -Z unstable-options
    )
fi

set -x

# Build documentation.
cargo "${toolchain}" doc --no-deps --all --all-features

# Run tests.
cargo "${toolchain}" test --all --all-features
