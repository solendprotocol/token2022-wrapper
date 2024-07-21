#!/bin/bash -e
ROOT=$(git rev-parse --show-toplevel)
cargo build-sbf
(cd $ROOT/shank-solita-scripts && yarn && node generateIdl.js)