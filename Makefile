SHELL := /usr/bin/env bash

export ORC_MAIN_PORT ?= 8028
export ORC_PEER_PORT ?= 8029
export ORC_MODALITY_ONE_PORT ?= 8021
export ORC_MODALITY_TWO_PORT ?= 8022

export ORC_MAIN_ADDRESS ?= http://localhost:${ORC_MAIN_PORT}
export ORC_PEER_ADDRESS ?= http://localhost:${ORC_PEER_PORT}
export ORC_MODALITY_ONE_ADDRESS ?= http://localhost:${ORC_MODALITY_ONE_PORT}
export ORC_MODALITY_TWO_ADDRESS ?= http://localhost:${ORC_MODALITY_TWO_PORT}

export ORC_ORTHANC_USERNAME ?= orthanc
export ORC_ORTHANC_PASSWORD ?= orthanc
export ORC_DATAFILES_PATH ?= ./tests/data/dicom

export DINO_SCP_HOST ?= 0.0.0.0
export DINO_SCP_PORT ?= 5252
export DINO_SCP_AET ?= DINO


build:
	cargo build

doc:
	cargo doc --no-deps

serve_doc: doc
	python -m http.server -b 127.0.0.1 -d target/doc 9001

clean: cleanup_orthanc stop_services
	cargo clean

test: unit_test integration_test e2e_test

unit_test:
	cargo test --lib -- --show-output ${TEST}

integration_test:
	cargo test --test client -- --test-threads=1 --show-output ${TEST}

e2e_test: reset_orthanc
	cargo test --test e2e -- --test-threads=1 --show-output ${TEST}

unit_test_coverage: install_tarpaulin
	cargo tarpaulin --lib --verbose --ignore-tests --all-features --workspace --timeout 120 --out Xml

integration_test_coverage: install_tarpaulin
	cargo tarpaulin --test client --verbose --ignore-tests --all-features --workspace --timeout 120 --out Xml -- --test-threads=1

e2e_test_coverage: install_tarpaulin reset_orthanc
	cargo tarpaulin --test e2e --verbose --ignore-tests --all-features --workspace --timeout 120 --out Xml -- --test-threads=1

install_tarpaulin:
	cargo install --version 0.16.0 cargo-tarpaulin

cleanup_orthanc:
	./scripts/cleanup_orthanc.sh

populate_orthanc:
	./scripts/populate_orthanc.sh

reset_orthanc: cleanup_orthanc populate_orthanc

start_services:
	docker-compose pull
	docker-compose up -d

stop_services:
	docker-compose down

release:
	cargo release ${VERSION}
