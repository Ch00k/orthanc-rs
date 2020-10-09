#!/usr/bin/env bash

set -e

source cleanup_orthanc.sh
trap cleanup EXIT

./populate_orthanc.sh
sleep 2
RUST_TEST_THREADS=1 cargo test --test integration -- $@
