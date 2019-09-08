#!/bin/bash

# A script to run compile tests with the same condition of the checks done by CI.
#
# Usage
#
# ```sh
# . ./compiletest.sh
# ```

rm -rf target/debug/deps/libpin_project* && RUSTFLAGS='--cfg compiletest --cfg pin_project_show_unpin_struct' cargo +nightly test -p pin-project --all-features --test compiletest
