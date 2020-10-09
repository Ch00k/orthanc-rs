#!/usr/bin/env bash

set -e

source cleanup_orthanc.sh
trap cleanup EXIT

./populate_orthanc.sh
sleep 2

cargo install cargo-tarpaulin
RUST_TEST_THREADS=1 cargo tarpaulin --lib --verbose --ignore-tests --all-features --workspace --timeout 120 --out Xml
