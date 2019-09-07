#!/bin/bash

# A script to run compile tests with the same condition that pin-project executes with CI.

rm -rf target/debug/deps/libpin_project* && RUSTFLAGS='--cfg compiletest --cfg pin_project_show_unpin_struct' cargo test -p pin-project --all-features --test compiletest
