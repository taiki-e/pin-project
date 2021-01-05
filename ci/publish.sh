#!/bin/bash

set -euo pipefail
IFS=$'\n\t'

cd "$(cd "$(dirname "${0}")" && pwd)"/..

set -x

cd pin-project-internal
cargo publish
cd ..

sleep 30
cargo publish
